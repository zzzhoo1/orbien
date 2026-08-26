use crate::connector::{build_connector, Connector};
use crate::session_id;
use crate::tunnel::TunnelManager;
use anyhow::{anyhow, Result};
use orbien_core::auth;
use orbien_core::config::ClientConfig;
use orbien_core::msg::{
    self, Login, Message, MessageReadError, NewDataConn, NewTunnel, Ping,
};
use orbien_core::transport::DynStream;
use orbien_core::VERSION;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::{interval, sleep};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum SessionEnd {
    Disconnected { session_id: String },
    Kicked { session_id: String, reason: String },
}

type CtrlRead = ReadHalf<DynStream>;
type CtrlWrite = WriteHalf<DynStream>;
type OnTunnelRemote = Arc<dyn Fn(String, String) + Send + Sync>;

pub struct Control {
    cfg: ClientConfig,
    session_id: String,
    reader: Mutex<CtrlRead>,
    writer: Mutex<CtrlWrite>,
    tunnels: TunnelManager,
    connector: Arc<dyn Connector>,
    cancel: CancellationToken,
    data_tasks: Mutex<JoinSet<()>>,
    on_tunnel_remote: OnTunnelRemote,
    last_pong_unix: AtomicI64,
}

impl Control {
    pub async fn start(
        cfg: &ClientConfig,
        previous_session_id: String,
        config_path: &Path,
        parent_cancel: CancellationToken,
        on_connected: impl FnOnce(),
        on_tunnel_remote: OnTunnelRemote,
    ) -> Result<SessionEnd> {
        let session_cancel = parent_cancel.child_token();
        let connector = build_connector(cfg).await?;
        let mut stream = connector.open().await?;
        tracing::info!(
            endpoint = %cfg.server_endpoint(),
            protocol = %cfg.transport.protocol,
            tcp_mux = cfg.uses_yamux(),
            "control stream opened"
        );

        let timestamp = now_secs();
        let auth_digest = auth::compute_auth_digest(&cfg.auth.token, timestamp);
        let login = Login {
            version: VERSION.into(),
            hostname: hostname(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            user: cfg.user.clone(),
            auth_digest,
            timestamp,
            session_id: previous_session_id,
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

        tracing::info!(session_id = %resp.session_id, "login ok");
        if let Err(e) = session_id::save(config_path, &resp.session_id) {
            tracing::warn!(error = %e, "failed to persist session_id");
        }

        let (reader, writer) = tokio::io::split(stream);
        let ctl = Arc::new(Control {
            cfg: cfg.clone(),
            session_id: resp.session_id.clone(),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            tunnels: TunnelManager::from_config(cfg)?,
            connector,
            cancel: session_cancel.clone(),
            data_tasks: Mutex::new(JoinSet::new()),
            on_tunnel_remote,
            last_pong_unix: AtomicI64::new(now_secs()),
        });

        ctl.register_all_tunnels().await?;
        on_connected();

        let hb = Arc::clone(&ctl);
        let hb_cancel = session_cancel.clone();
        let heartbeat = tokio::spawn(async move {
            tokio::select! {
                _ = hb_cancel.cancelled() => {}
                _ = hb.heartbeat_loop() => {}
            }
        });

        let to = Arc::clone(&ctl);
        let to_cancel = session_cancel.clone();
        let timeout_watch = tokio::spawn(async move {
            tokio::select! {
                _ = to_cancel.cancelled() => {}
                _ = to.heartbeat_timeout_loop() => {}
            }
        });

        let result = ctl.clone().reader_loop().await;
        ctl.shutdown().await;
        heartbeat.abort();
        timeout_watch.abort();
        let _ = heartbeat.await;
        let _ = timeout_watch.await;

        match result {
            Ok(ReaderEnd::Kicked(reason)) => Ok(SessionEnd::Kicked {
                session_id: resp.session_id,
                reason,
            }),
            Ok(ReaderEnd::Closed) => Ok(SessionEnd::Disconnected {
                session_id: resp.session_id,
            }),
            Err(e) => Err(e),
        }
    }

    async fn shutdown(&self) {
        self.cancel.cancel();
        {
            let mut writer = self.writer.lock().await;
            let _ = writer.shutdown().await;
        }
        let mut tasks = self.data_tasks.lock().await;
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }

    async fn register_all_tunnels(&self) -> Result<()> {
        for p in &self.cfg.tunnels {
            let (local_ip, local_port) = p.service_host_port()?;
            if p.requires_local_service() && local_port == 0 {
                return Err(anyhow!(
                    "tunnel `{}` requires service = \"host:port\" (local backend)",
                    p.name
                ));
            }
            if p.remote_port == 0 && matches!(p.protocol.as_str(), "tcp" | "udp") {
                return Err(anyhow!(
                    "tunnel `{}` type {} requires remotePort > 0",
                    p.name,
                    p.protocol
                ));
            }
            let msg = match p.protocol.as_str() {
                "tcp" => Message::NewTunnel(new_tunnel_base(
                    &p.name,
                    "tcp",
                    p.remote_port as i32,
                    &local_ip,
                    local_port,
                    &p.transport,
                    p.max_connections,
                    |_| {},
                )),
                "udp" => Message::NewTunnel(new_tunnel_base(
                    &p.name,
                    "udp",
                    p.remote_port as i32,
                    &local_ip,
                    local_port,
                    &p.transport,
                    p.max_connections,
                    |_| {},
                )),
                "http" => Message::NewTunnel(new_tunnel_base(
                    &p.name,
                    "http",
                    0,
                    &local_ip,
                    local_port,
                    &p.transport,
                    p.max_connections,
                    |np| {
                        np.domains = p.domains.clone();
                        np.locations = p.locations.clone();
                        np.basic_auth_user = p.basic_auth_user.clone();
                        np.basic_auth_password = p.basic_auth_password.clone();
                        np.host_header_rewrite = p.host_header_rewrite.clone();
                        np.route_by_http_user = p.route_by_http_user.clone();
                    },
                )),
                "https" => Message::NewTunnel(new_tunnel_base(
                    &p.name,
                    "https",
                    0,
                    &local_ip,
                    local_port,
                    &p.transport,
                    p.max_connections,
                    |np| {
                        np.domains = p.domains.clone();
                    },
                )),
                other => {
                    tracing::warn!(name = %p.name, protocol = %other, "skip unsupported tunnel protocol");
                    continue;
                }
            };
            let mut writer = self.writer.lock().await;
            msg::write_msg(&mut *writer, &msg).await?;
            match p.protocol.as_str() {
                "tcp" => tracing::info!(
                    name = %p.name,
                    service = %p.service,
                    remote_port = p.remote_port,
                    "sent NewTunnel"
                ),
                "udp" => tracing::info!(
                    name = %p.name,
                    service = %p.service,
                    remote_port = p.remote_port,
                    "sent NewTunnel udp"
                ),
                "http" => tracing::info!(
                    name = %p.name,
                    service = %p.service,
                    domains = ?p.domains,
                    "sent NewTunnel http"
                ),
                "https" => tracing::info!(
                    name = %p.name,
                    service = %p.service,
                    domains = ?p.domains,
                    "sent NewTunnel https"
                ),
                _ => {}
            }
        }
        Ok(())
    }

    async fn reader_loop(self: Arc<Self>) -> Result<ReaderEnd> {
        loop {
            if self.cancel.is_cancelled() {
                return Ok(ReaderEnd::Closed);
            }

            let msg = tokio::select! {
                _ = self.cancel.cancelled() => {
                    return Ok(ReaderEnd::Closed);
                }
                msg = async {
                    let mut reader = self.reader.lock().await;
                    msg::read_msg(&mut *reader).await
                } => {
                    match msg {
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
                }
            };

            match msg {
                Message::KickOut(k) => {
                    tracing::warn!(reason = %k.reason, "kicked by server — will exit");
                    return Ok(ReaderEnd::Kicked(k.reason));
                }
                Message::ReqDataConn(_) => {
                    let ctl = Arc::clone(&self);
                    let cancel = self.cancel.clone();
                    self.data_tasks.lock().await.spawn(async move {
                        tokio::select! {
                            _ = cancel.cancelled() => {}
                            res = ctl.handle_req_data_conn() => {
                                if let Err(e) = res {
                                    tracing::error!(error = %e, "data conn failed");
                                }
                            }
                        }
                    });
                }
                Message::NewTunnelResp(resp) => {
                    if resp.error.is_empty() {
                        let remote = normalize_remote_addr(&self.cfg.server, &resp.remote_addr);
                        tracing::info!(
                            name = %resp.tunnel_name,
                            remote = %remote,
                            "tunnel started"
                        );
                        (self.on_tunnel_remote)(resp.tunnel_name.clone(), remote);
                    } else {
                        tracing::error!(
                            name = %resp.tunnel_name,
                            error = %resp.error,
                            "tunnel start failed"
                        );
                    }
                }
                Message::Pong(_) => {
                    self.last_pong_unix.store(now_secs(), Ordering::Relaxed);
                    tracing::trace!("pong");
                }
                other => {
                    tracing::warn!(ty = other.type_byte(), "ignored message");
                }
            }
        }
    }

    async fn heartbeat_loop(self: Arc<Self>) {
        let secs = self.effective_ping_interval();
        if secs <= 0 {
            tracing::debug!("app heartbeat disabled");
            std::future::pending::<()>().await;
            return;
        }
        let mut tick = interval(Duration::from_secs(secs as u64));
        tick.tick().await;
        loop {
            if self.cancel.is_cancelled() {
                break;
            }
            tick.tick().await;
            let timestamp = now_secs();
            let ping = Ping {
                auth_digest: auth::compute_auth_digest(&self.cfg.auth.token, timestamp),
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

    async fn heartbeat_timeout_loop(self: Arc<Self>) {
        let timeout = self.effective_pong_timeout();
        if timeout <= 0 {
            std::future::pending::<()>().await;
            return;
        }
        loop {
            if self.cancel.is_cancelled() {
                break;
            }
            sleep(Duration::from_secs(1)).await;
            let last = self.last_pong_unix.load(Ordering::Relaxed);
            let now = now_secs();
            if last > 0 && now.saturating_sub(last) > timeout {
                tracing::warn!(timeout_secs = timeout, "heartbeat timeout");
                self.cancel.cancel();
                break;
            }
        }
    }

    fn effective_ping_interval(&self) -> i64 {
        let hb = self.cfg.transport.heartbeat_interval;
        if hb > 0 {
            return hb;
        }
        if self.cfg.transport.tcp_mux {
            let mux_ka = self.cfg.transport.mux_keepalive_secs;
            if mux_ka > 0 {
                return mux_ka;
            }
        }
        -1
    }

    fn effective_pong_timeout(&self) -> i64 {
        let hb_to = self.cfg.transport.heartbeat_timeout;
        if hb_to > 0 {
            return hb_to;
        }
        if self.cfg.transport.heartbeat_interval <= 0 && self.cfg.transport.tcp_mux {
            let mux_ka = self.cfg.transport.mux_keepalive_secs;
            if mux_ka > 0 {
                return mux_ka.saturating_mul(3);
            }
        }
        -1
    }

    async fn handle_req_data_conn(self: Arc<Self>) -> Result<()> {
        let mut data = self.connector.open().await?;

        let timestamp = now_secs();
        msg::write_msg(
            &mut data,
            &Message::NewDataConn(NewDataConn {
                session_id: self.session_id.clone(),
                auth_digest: auth::compute_auth_digest(&self.cfg.auth.token, timestamp),
                timestamp,
            }),
        )
        .await?;

        let start = tokio::select! {
            _ = self.cancel.cancelled() => {
                return Ok(());
            }
            msg = msg::read_msg(&mut data) => {
                match msg? {
                    Message::StartDataConn(s) => s,
                    other => {
                        return Err(anyhow!("expected StartDataConn, got {}", other.type_byte()))
                    }
                }
            }
        };

        if !start.error.is_empty() {
            return Err(anyhow!("StartDataConn error: {}", start.error));
        }

        self.tunnels.handle_data_conn(&start, data).await
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

fn omit_client_side(side: &str) -> String {
    match side.trim().to_ascii_lowercase().as_str() {
        "" | "client" => String::new(),
        other => other.to_string(),
    }
}

fn normalize_remote_addr(server_addr: &str, remote_addr: &str) -> String {
    let remote = remote_addr.trim();
    if remote.is_empty() {
        return String::new();
    }
    if let Some(port) = remote.strip_prefix(':') {
        let host = server_addr.trim();
        let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
        if !host.is_empty() && !port.is_empty() && !host.contains(':') {
            return format!("{host}:{port}");
        }
        if !host.is_empty() && !port.is_empty() {
            return format!("{host}:{port}");
        }
    }
    remote.to_string()
}

#[allow(clippy::too_many_arguments)]
fn new_tunnel_base(
    name: &str,
    protocol: &str,
    remote_port: i32,
    local_ip: &str,
    local_port: u16,
    transport: &orbien_core::config::TunnelTransportConfig,
    max_connections: usize,
    extra: impl FnOnce(&mut NewTunnel),
) -> NewTunnel {
    let mut np = NewTunnel {
        tunnel_name: name.into(),
        protocol: protocol.into(),
        remote_port,
        local_ip: local_ip.into(),
        local_port: i32::from(local_port),
        domains: Vec::new(),
        locations: Vec::new(),
        basic_auth_user: String::new(),
        basic_auth_password: String::new(),
        host_header_rewrite: String::new(),
        headers: Default::default(),
        response_headers: Default::default(),
        route_by_http_user: String::new(),
        bandwidth: transport.bandwidth,
        bandwidth_limit_side: omit_client_side(&transport.bandwidth_limit_side),
        max_connections,
    };
    extra(&mut np);
    np
}
