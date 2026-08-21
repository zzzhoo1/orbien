use crate::connector::{build_connector, Connector};
use crate::proxy::ProxyManager;
use crate::run_id;
use anyhow::{anyhow, Result};
use orbien_core::auth;
use orbien_core::config::ClientConfig;
use orbien_core::msg::{self, Login, Message, MessageReadError, NewProxy, NewWorkConn, Ping};
use std::path::Path;

#[derive(Debug)]
pub enum SessionEnd {
    Disconnected { run_id: String },
    Kicked { run_id: String, reason: String },
}
use orbien_core::transport::DynStream;
use orbien_core::VERSION;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::Mutex;
use tokio::time::interval;

type CtrlRead = ReadHalf<DynStream>;
type CtrlWrite = WriteHalf<DynStream>;

pub struct Control {
    cfg: ClientConfig,
    run_id: String,
    reader: Mutex<CtrlRead>,
    writer: Mutex<CtrlWrite>,
    proxies: ProxyManager,
    connector: Arc<dyn Connector>,
}

impl Control {
    pub async fn start(
        cfg: &ClientConfig,
        previous_run_id: String,
        config_path: &Path,
    ) -> Result<SessionEnd> {
        let connector = build_connector(cfg).await?;
        let mut stream = connector.open().await?;
        tracing::info!(
            endpoint = %cfg.server_endpoint(),
            protocol = %cfg.transport.protocol,
            tcp_mux = cfg.uses_yamux(),
            "control stream opened"
        );

        let timestamp = now_secs();
        let privilege_key = auth::get_auth_key(&cfg.auth.token, timestamp);
        let login = Login {
            version: VERSION.into(),
            hostname: hostname(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            user: cfg.user.clone(),
            privilege_key,
            timestamp,
            run_id: previous_run_id,
            pool_count: cfg.transport.pool_count,
        };
        tracing::info!(
            hostname = %login.hostname,
            os = %login.os,
            arch = %login.arch,
            user = %login.user,
            "login identity"
        );

        msg::write_msg(&mut stream, &Message::Login(login)).await?;

        // Explicit match so an EOF here (e.g. server rejects due to TLS/auth
        // mismatch and closes the connection) produces an actionable message
        // instead of a raw rustls "peer closed connection without close_notify".
        let resp = match msg::read_msg(&mut stream).await {
            Ok(Message::LoginResp(r)) => r,
            Ok(other) => {
                return Err(anyhow!(
                    "expected LoginResp, got message type {}; check server version",
                    other.type_byte()
                ));
            }
            Err(e) => {
                return Err(anyhow!(
                    "server closed connection before sending LoginResp: {e}; \
                     verify transport.tls, tcpMux, protocol, token, and bind_port match on both sides"
                ));
            }
        };

        if !resp.error.is_empty() {
            return Err(anyhow!("login rejected by server: {}", resp.error));
        }

        tracing::info!(run_id = %resp.run_id, "login ok");
        if let Err(e) = run_id::save(config_path, &resp.run_id) {
            tracing::warn!(error = %e, "failed to persist run_id");
        }

        let (reader, writer) = tokio::io::split(stream);
        let ctl = Arc::new(Control {
            cfg: cfg.clone(),
            run_id: resp.run_id.clone(),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            proxies: ProxyManager::from_config(cfg)?,
            connector,
        });

        ctl.register_all_proxies().await?;

        let hb = Arc::clone(&ctl);
        let heartbeat = tokio::spawn(async move { hb.heartbeat_loop().await });

        let result = ctl.reader_loop().await;
        heartbeat.abort();
        let _ = heartbeat.await;
        match result {
            Ok(ReaderEnd::Kicked(reason)) => Ok(SessionEnd::Kicked {
                run_id: resp.run_id,
                reason,
            }),
            Ok(ReaderEnd::Closed) => Ok(SessionEnd::Disconnected {
                run_id: resp.run_id,
            }),
            Err(e) => Err(e),
        }
    }

    async fn register_all_proxies(&self) -> Result<()> {
        for p in &self.cfg.proxies {
            let msg = match p.proxy_type.as_str() {
                "tcp" => Message::NewProxy(new_proxy_base(
                    &p.name,
                    "tcp",
                    p.remote_port as i32,
                    &p.local_ip,
                    p.local_port,
                    &p.transport,
                    p.max_connections,
                    |np| {
                        np.custom_domains = Vec::new();
                    },
                )),
                "udp" => Message::NewProxy(new_proxy_base(
                    &p.name,
                    "udp",
                    p.remote_port as i32,
                    &p.local_ip,
                    p.local_port,
                    &p.transport,
                    p.max_connections,
                    |_| {},
                )),
                "http" => Message::NewProxy(new_proxy_base(
                    &p.name,
                    "http",
                    0,
                    &p.local_ip,
                    p.local_port,
                    &p.transport,
                    p.max_connections,
                    |np| {
                        np.custom_domains = p.custom_domains.clone();
                        np.subdomain = p.subdomain.clone();
                        np.locations = p.locations.clone();
                        np.http_user = p.http_user.clone();
                        np.http_pwd = p.http_password.clone();
                        np.host_header_rewrite = p.host_header_rewrite.clone();
                        np.route_by_http_user = p.route_by_http_user.clone();
                    },
                )),
                "https" => Message::NewProxy(new_proxy_base(
                    &p.name,
                    "https",
                    0,
                    &p.local_ip,
                    p.local_port,
                    &p.transport,
                    p.max_connections,
                    |np| {
                        np.custom_domains = p.custom_domains.clone();
                        np.subdomain = p.subdomain.clone();
                    },
                )),
                other => {
                    tracing::warn!(name = %p.name, ty = %other, "skip unsupported proxy type");
                    continue;
                }
            };
            let mut writer = self.writer.lock().await;
            msg::write_msg(&mut *writer, &msg).await?;
            match p.proxy_type.as_str() {
                "tcp" => tracing::info!(
                    name = %p.name,
                    local = %format!("{}:{}", p.local_ip, p.local_port),
                    remote_port = p.remote_port,
                    "sent NewProxy"
                ),
                "udp" => tracing::info!(
                    name = %p.name,
                    local = %format!("{}:{}", p.local_ip, p.local_port),
                    remote_port = p.remote_port,
                    "sent NewProxy udp"
                ),
                "http" => tracing::info!(
                    name = %p.name,
                    local = %format!("{}:{}", p.local_ip, p.local_port),
                    domains = ?p.custom_domains,
                    subdomain = %p.subdomain,
                    "sent NewProxy http"
                ),
                "https" => tracing::info!(
                    name = %p.name,
                    local = %format!("{}:{}", p.local_ip, p.local_port),
                    domains = ?p.custom_domains,
                    subdomain = %p.subdomain,
                    "sent NewProxy https (SNI passthrough)"
                ),
                _ => {}
            }
        }
        Ok(())
    }

    async fn reader_loop(self: Arc<Self>) -> Result<ReaderEnd> {
        loop {
            let msg = {
                let mut reader = self.reader.lock().await;
                match msg::read_msg(&mut *reader).await {
                    Ok(m) => m,
                    // EOF without TLS close_notify: treat as clean disconnect.
                    // Per rustls docs, this is safe when the application protocol
                    // (our framed msgpack) already provides message-length framing.
                    Err(e) if is_unexpected_eof(&e) => {
                        tracing::debug!(
                            "control stream closed (unexpected EOF treated as clean disconnect)"
                        );
                        return Ok(ReaderEnd::Closed);
                    }
                    Err(_) => return Ok(ReaderEnd::Closed),
                }
            };

            match msg {
                Message::KickOut(k) => {
                    tracing::warn!(reason = %k.reason, "kicked by server — will exit");
                    return Ok(ReaderEnd::Kicked(k.reason));
                }
                Message::ReqWorkConn(_) => {
                    let ctl = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = ctl.handle_req_work_conn().await {
                            tracing::error!(error = %e, "work tunnel failed");
                        }
                    });
                }
                Message::NewProxyResp(resp) => {
                    if resp.error.is_empty() {
                        tracing::info!(
                            name = %resp.proxy_name,
                            remote = %resp.remote_addr,
                            "proxy started"
                        );
                    } else {
                        tracing::error!(
                            name = %resp.proxy_name,
                            error = %resp.error,
                            "proxy start failed"
                        );
                    }
                }
                Message::Pong(_) => {
                    tracing::trace!("pong");
                }
                other => {
                    tracing::warn!(ty = other.type_byte(), "ignored message");
                }
            }
        }
    }

    async fn heartbeat_loop(self: Arc<Self>) {
        let secs = self.cfg.transport.heartbeat_interval;
        if secs <= 0 {
            tracing::debug!("app heartbeat disabled (tcpMux / heartbeatInterval<=0)");
            std::future::pending::<()>().await;
            return;
        }
        let mut tick = interval(Duration::from_secs(secs as u64));
        tick.tick().await;
        loop {
            tick.tick().await;
            let timestamp = now_secs();
            let ping = Ping {
                privilege_key: auth::get_auth_key(&self.cfg.auth.token, timestamp),
                timestamp,
            };
            let mut writer = self.writer.lock().await;
            if msg::write_msg(&mut *writer, &Message::Ping(ping))
                .await
                .is_err()
            {
                break;
            }
        }
    }

    async fn handle_req_work_conn(self: Arc<Self>) -> Result<()> {
        let mut work = self.connector.open().await?;

        let timestamp = now_secs();
        msg::write_msg(
            &mut work,
            &Message::NewWorkConn(NewWorkConn {
                run_id: self.run_id.clone(),
                privilege_key: auth::get_auth_key(&self.cfg.auth.token, timestamp),
                timestamp,
            }),
        )
        .await?;

        let start = match msg::read_msg(&mut work).await? {
            Message::StartWorkConn(s) => s,
            other => return Err(anyhow!("expected StartWorkConn, got {}", other.type_byte())),
        };

        if !start.error.is_empty() {
            return Err(anyhow!("StartWorkConn error: {}", start.error));
        }

        self.proxies.handle_work_conn(&start, work).await
    }
}

enum ReaderEnd {
    Closed,
    Kicked(String),
}

/// Returns true if the error represents an EOF without TLS close_notify.
/// Per rustls documentation, this can be safely treated as a clean shutdown
/// when the application protocol uses length-framed messages (as we do).
fn is_unexpected_eof(e: &MessageReadError) -> bool {
    use std::io::ErrorKind;
    match e {
        MessageReadError::Io(io_err) => io_err.kind() == ErrorKind::UnexpectedEof,
        _ => {
            let msg = e.to_string();
            msg.contains("unexpected eof")
                || msg.contains("UnexpectedEof")
                || msg.contains("close_notify")
        }
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn hostname() -> String {
    if let Ok(name) = hostname::get() {
        let s = name.to_string_lossy().trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }

    ["HOSTNAME", "COMPUTERNAME", "HOST"]
        .into_iter()
        .find_map(|k| std::env::var(k).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn omit_client_mode(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        "" | "client" => String::new(),
        other => other.to_string(),
    }
}

fn new_proxy_base(
    name: &str,
    proxy_type: &str,
    remote_port: i32,
    local_ip: &str,
    local_port: u16,
    transport: &orbien_core::config::ProxyTransportConfig,
    max_connections: usize,
    extra: impl FnOnce(&mut NewProxy),
) -> NewProxy {
    let mut np = NewProxy {
        proxy_name: name.into(),
        proxy_type: proxy_type.into(),
        remote_port,
        local_ip: local_ip.into(),
        local_port: i32::from(local_port),
        custom_domains: Vec::new(),
        subdomain: String::new(),
        locations: Vec::new(),
        http_user: String::new(),
        http_pwd: String::new(),
        host_header_rewrite: String::new(),
        headers: Default::default(),
        response_headers: Default::default(),
        route_by_http_user: String::new(),
        bandwidth_limit: transport.bandwidth_limit.clone(),
        bandwidth_limit_mode: omit_client_mode(&transport.bandwidth_limit_mode),
        max_connections,
    };
    extra(&mut np);
    np
}
