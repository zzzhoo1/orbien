use crate::access::AccessPolicy;
use crate::control::Control;
use crate::metrics::{MemMetrics, ServerMetrics};
use crate::proxy::{run_vhost_http_listener, run_vhost_https_listener, HttpVhost, HttpsVhost};
use anyhow::{anyhow, Result};
use orbien_core::auth;
use orbien_core::config::ServerConfig;
use orbien_core::msg::{self, Login, LoginResp, Message, NewWorkConn};
use orbien_core::transport::{self, boxed_stream, DynStream};
use orbien_core::VERSION;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinSet;
use uuid::Uuid;

struct OfflineClientRecord {
    run_id: String,
    user: String,
    hostname: String,
    os: String,
    arch: String,
    client_ip: String,
    version: String,
    proxy_count: usize,
    disconnected_at: Instant,
}

pub struct Service {
    cfg: ServerConfig,
    access: Arc<AccessPolicy>,
    controls: Arc<Mutex<HashMap<String, Arc<Control>>>>,
    offline_clients: Arc<Mutex<HashMap<String, OfflineClientRecord>>>,
    http_vhost: Option<Arc<HttpVhost>>,
    https_vhost: Option<Arc<HttpsVhost>>,
    tls_config: Arc<rustls::ServerConfig>,
    metrics: Arc<MemMetrics>,
}

impl Service {
    pub fn new(cfg: ServerConfig) -> Result<Self> {
        let access = Arc::new(AccessPolicy::from_server_config(&cfg)?);
        let http_vhost = if cfg.vhost_http_enabled() {
            Some(Arc::new(HttpVhost::new(cfg.vhost_http_port)))
        } else {
            None
        };
        let https_vhost = if cfg.vhost_https_enabled() {
            Some(Arc::new(HttpsVhost::new(cfg.vhost_https_port)))
        } else {
            None
        };
        let tls = &cfg.transport.tls;
        let tls_config =
            transport::new_server_tls_config(&tls.cert_file, &tls.key_file, &tls.trusted_ca_file)?;
        if tls.force {
            tracing::info!("transport.tls.force=true — non-TLS control connections rejected");
        }
        Ok(Self {
            cfg,
            access,
            controls: Arc::new(Mutex::new(HashMap::new())),
            offline_clients: Arc::new(Mutex::new(HashMap::new())),
            http_vhost,
            https_vhost,
            tls_config,
            metrics: MemMetrics::new(),
        })
    }

    pub async fn run(self) -> Result<()> {
        let this = Arc::new(self);

        if this.cfg.quic_enabled()
            && this.cfg.kcp_enabled()
            && this.cfg.quic_bind_port == this.cfg.kcp_bind_port
        {
            return Err(anyhow!(
                "quicBindPort and kcpBindPort both use UDP and must differ (got {})",
                this.cfg.quic_bind_port
            ));
        }

        let tcp_addr = format!("{}:{}", this.cfg.bind_addr, this.cfg.bind_port);
        let tcp_listener = TcpListener::bind(&tcp_addr).await?;
        tracing::info!(
            %tcp_addr,
            ws_path = transport::ORBIEN_WEBSOCKET_PATH,
            tcp_mux = this.cfg.transport.tcp_mux,
            "tcp/websocket control/work listener ready"
        );

        let vhost_shutdown = Arc::new(Notify::new());
        let mut set = JoinSet::new();

        if let Some(ref vhost) = this.http_vhost {
            let bind = this.cfg.proxy_bind_addr.clone();
            let port = this.cfg.vhost_http_port;
            let vhost = Arc::clone(vhost);
            let access = Arc::clone(&this.access);
            let shutdown = Arc::clone(&vhost_shutdown);
            set.spawn(
                async move { run_vhost_http_listener(bind, port, vhost, access, shutdown).await },
            );
        }

        if let Some(ref vhost) = this.https_vhost {
            let bind = this.cfg.proxy_bind_addr.clone();
            let port = this.cfg.vhost_https_port;
            let vhost = Arc::clone(vhost);
            let access = Arc::clone(&this.access);
            let shutdown = Arc::clone(&vhost_shutdown);
            set.spawn(async move {
                run_vhost_https_listener(bind, port, vhost, access, shutdown).await
            });
        }

        if this.cfg.quic_enabled() {
            let quic_addr: SocketAddr =
                format!("{}:{}", this.cfg.bind_addr, this.cfg.quic_bind_port)
                    .parse()
                    .map_err(|e| anyhow!("invalid quic bind addr: {e}"))?;
            let endpoint = transport::build_server_endpoint(
                quic_addr,
                this.cfg.transport.quic.keepalive(),
                this.cfg.transport.quic.idle_timeout(),
                this.cfg.transport.quic.max_incoming_streams,
                &this.cfg.transport.tls.cert_file,
                &this.cfg.transport.tls.key_file,
                &this.cfg.transport.tls.trusted_ca_file,
            )?;
            tracing::info!(%quic_addr, "quic control/work listener ready");
            let svc = Arc::clone(&this);
            set.spawn(async move { svc.run_quic(endpoint).await });
        }

        if this.cfg.kcp_enabled() {
            let kcp_addr: SocketAddr = format!("{}:{}", this.cfg.bind_addr, this.cfg.kcp_bind_port)
                .parse()
                .map_err(|e| anyhow!("invalid kcp bind addr: {e}"))?;
            let listener = transport::bind_kcp_listener(kcp_addr).await?;
            tracing::info!(
                %kcp_addr,
                tcp_mux = this.cfg.transport.tcp_mux,
                "kcp control/work listener ready"
            );
            let svc = Arc::clone(&this);
            set.spawn(async move { svc.run_kcp(listener).await });
        }

        if this.cfg.web_server.enabled() {
            let web_cfg = this.cfg.web_server.clone();
            let svc = Arc::clone(&this);
            set.spawn(async move { crate::dashboard::run(svc, web_cfg).await });
        }

        let svc = Arc::clone(&this);
        set.spawn(async move { svc.run_tcp(tcp_listener).await });

        let first = set
            .join_next()
            .await
            .ok_or_else(|| anyhow!("no listener tasks"))?;
        vhost_shutdown.notify_waiters();
        set.abort_all();
        while set.join_next().await.is_some() {}

        match first {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(e) if e.is_cancelled() => Ok(()),
            Err(e) => Err(anyhow!("listener task join: {e}")),
        }
    }

    async fn run_tcp(self: Arc<Self>, listener: TcpListener) -> Result<()> {
        loop {
            let (stream, peer) = listener.accept().await?;
            let svc = Arc::clone(&self);
            tokio::spawn(async move {
                if let Err(e) = svc.handle_tcp_or_websocket(stream, peer).await {
                    tracing::warn!(%peer, error = %e, "tcp/ws connection closed with error");
                }
            });
        }
    }

    async fn handle_tcp_or_websocket(
        self: Arc<Self>,
        stream: TcpStream,
        peer: SocketAddr,
    ) -> Result<()> {
        let mut peek_buf = [0u8; 16];
        let n = stream.peek(&mut peek_buf).await.unwrap_or(0);
        let physical = if transport::is_websocket_http_request(&peek_buf[..n]) {
            tracing::debug!(%peer, transport = "websocket", "upgrade");
            transport::accept_websocket(stream).await?
        } else {
            tracing::debug!(%peer, transport = "tcp", "incoming connection");
            boxed_stream(stream)
        };
        let physical = transport::check_and_enable_tls(
            physical,
            Arc::clone(&self.tls_config),
            self.cfg.transport.tls.force,
        )
        .await?;
        self.handle_physical(physical, peer).await
    }

    async fn handle_physical(self: Arc<Self>, physical: DynStream, peer: SocketAddr) -> Result<()> {
        if self.cfg.transport.tcp_mux {
            tracing::debug!(%peer, "yamux server session started");
            let svc = Arc::clone(&self);
            transport::serve_yamux_session(physical, move |stream| {
                let svc = Arc::clone(&svc);
                tokio::spawn(async move {
                    if let Err(e) = svc.handle_connection(stream, peer).await {
                        tracing::debug!(error = %e, "yamux stream closed with error");
                    }
                });
            })
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unknown version: 111") || msg.contains("unknown version: 119") {
                    anyhow!(
                        "yamux session {peer}: {e} — transport.tcpMux mismatch: server expects yamux \
                         (tcpMux=true) but peer sent a raw control frame (Login 'o'=111 / NewWorkConn 'w'=119). \
                         Set the same tcpMux on orbien and orbien-server, then restart both."
                    )
                } else {
                    anyhow!("yamux session {peer}: {e}")
                }
            })
        } else {
            self.handle_connection(physical, peer).await
        }
    }

    async fn run_kcp(self: Arc<Self>, mut listener: kcp_tokio::KcpListener) -> Result<()> {
        loop {
            let (stream, peer) = transport::accept_kcp(&mut listener).await?;
            tracing::debug!(%peer, transport = "kcp", "incoming connection");
            let svc = Arc::clone(&self);
            tokio::spawn(async move {
                let result = async {
                    let stream = transport::check_and_enable_tls(
                        stream,
                        Arc::clone(&svc.tls_config),
                        svc.cfg.transport.tls.force,
                    )
                    .await?;
                    svc.handle_physical(stream, peer).await
                }
                .await;
                if let Err(e) = result {
                    tracing::warn!(%peer, error = %e, "kcp connection closed with error");
                }
            });
        }
    }

    async fn run_quic(self: Arc<Self>, endpoint: quinn::Endpoint) -> Result<()> {
        loop {
            let incoming = endpoint
                .accept()
                .await
                .ok_or_else(|| anyhow!("quic endpoint closed"))?;
            let svc = Arc::clone(&self);
            tokio::spawn(async move {
                match incoming.await {
                    Ok(conn) => {
                        let peer = conn.remote_address();
                        tracing::info!(%peer, "quic session accepted");
                        if let Err(e) = svc.handle_quic_connection(conn).await {
                            tracing::debug!(%peer, error = %e, "quic session ended");
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "quic accept failed"),
                }
            });
        }
    }

    async fn handle_quic_connection(self: Arc<Self>, conn: quinn::Connection) -> Result<()> {
        loop {
            let (send, recv) = conn.accept_bi().await?;
            let stream = transport::quic_bi(send, recv);
            let svc = Arc::clone(&self);
            let peer = conn.remote_address();
            tokio::spawn(async move {
                if let Err(e) = svc.handle_connection(stream, peer).await {
                    tracing::debug!(%peer, error = %e, "quic stream closed with error");
                }
            });
        }
    }

    async fn handle_connection(
        self: Arc<Self>,
        mut stream: DynStream,
        peer: SocketAddr,
    ) -> Result<()> {
        let first = msg::read_msg(&mut stream).await?;
        match first {
            Message::Login(login) => self.register_control(stream, login, peer).await,
            Message::NewWorkConn(nw) => self.register_work_conn(stream, nw).await,
            other => Err(anyhow!("unexpected first message: {:?}", other.type_byte())),
        }
    }

    async fn register_control(
        self: Arc<Self>,
        stream: DynStream,
        login: Login,
        peer: SocketAddr,
    ) -> Result<()> {
        if !auth::verify_login(&self.cfg.auth.token, &login.privilege_key, login.timestamp) {
            let mut stream = stream;
            let _ = msg::write_msg(
                &mut stream,
                &Message::LoginResp(LoginResp {
                    version: VERSION.into(),
                    run_id: String::new(),
                    error: "authorization failed".into(),
                }),
            )
            .await;
            return Err(anyhow!("authorization failed"));
        }

        if let Err(reason) = validate_login_fields(&login) {
            let mut stream = stream;
            let _ = msg::write_msg(
                &mut stream,
                &Message::LoginResp(LoginResp {
                    version: VERSION.into(),
                    run_id: String::new(),
                    error: reason.clone(),
                }),
            )
            .await;
            return Err(anyhow!(reason));
        }

        let run_id = if login.run_id.trim().is_empty() {
            short_run_id()
        } else {
            login.run_id.trim().to_string()
        };

        let stream = stream;

        tracing::info!(%run_id, %peer, pool = login.pool_count, "client logged in");

        let max_pool = self.cfg.transport.max_pool_count.max(0) as usize;
        let pool_count = (login.pool_count.max(0) as usize).min(max_pool);

        let client_ip = peer.ip().to_string();

        let control = Control::new(
            run_id.clone(),
            stream,
            self.cfg.clone(),
            pool_count,
            self.http_vhost.clone(),
            self.https_vhost.clone(),
            Arc::clone(&self.access),
            login.user.clone(),
            login.hostname.clone(),
            login.os.clone(),
            login.arch.clone(),
            login.version.clone(),
            client_ip,
            Arc::clone(&self.metrics),
        );
        let control = Arc::new(control);

        {
            let mut offline = self.offline_clients.lock().await;
            offline.remove(&run_id);
        }

        // #3 — single_client_per_user: kick any existing connection from the
        // same user (different run_id) before inserting the new one.
        if self.cfg.single_client_per_user && !login.user.trim().is_empty() {
            let to_kick: Vec<Arc<Control>> = {
                let map = self.controls.lock().await;
                map.values()
                    .filter(|c| !c.user.is_empty() && c.user == login.user && c.run_id != run_id)
                    .cloned()
                    .collect()
            };
            for old in to_kick {
                tracing::info!(
                    run_id = %old.run_id,
                    user = %old.user,
                    new_run_id = %run_id,
                    "single_client_per_user: kicking old session"
                );
                old.kick("replaced by newer login").await;
            }
        }

        let old = {
            let mut map = self.controls.lock().await;
            map.remove(&run_id)
        };

        if let Some(old) = old {
            tracing::info!(run_id = %run_id, "replacing existing session with same run_id");
            old.shutdown().await;
        }

        {
            let mut map = self.controls.lock().await;
            map.insert(run_id.clone(), Arc::clone(&control));
        }

        control
            .send_login_resp(LoginResp {
                version: VERSION.into(),
                run_id: run_id.clone(),
                error: String::new(),
            })
            .await?;

        self.metrics.new_client(&run_id);

        let controls = Arc::clone(&self.controls);
        let offline_clients = Arc::clone(&self.offline_clients);
        let metrics = Arc::clone(&self.metrics);
        let rid = run_id.clone();
        let result = Arc::clone(&control).run().await;
        control.shutdown().await;
        metrics.close_client();

        let proxy_count = control.proxy_count().await;
        let mut map = controls.lock().await;
        if map
            .get(&rid)
            .map(|c| Arc::ptr_eq(c, &control))
            .unwrap_or(false)
        {
            map.remove(&rid);
        }
        if !map.contains_key(&rid) {
            drop(map);
            let mut offline = offline_clients.lock().await;
            offline.insert(
                rid.clone(),
                OfflineClientRecord {
                    run_id: rid,
                    user: control.user.clone(),
                    hostname: control.hostname.clone(),
                    os: control.os.clone(),
                    arch: control.arch.clone(),
                    client_ip: control.client_ip.clone(),
                    version: control.version.clone(),
                    proxy_count,
                    disconnected_at: Instant::now(),
                },
            );
        }

        result
    }

    async fn register_work_conn(self: Arc<Self>, stream: DynStream, nw: NewWorkConn) -> Result<()> {
        if !auth::verify_login(&self.cfg.auth.token, &nw.privilege_key, nw.timestamp) {
            return Err(anyhow!("work conn authorization failed"));
        }

        let control = {
            let map = self.controls.lock().await;
            map.get(&nw.run_id).cloned()
        };
        match control {
            Some(c) => {
                c.push_work_conn(stream).await;
                Ok(())
            }
            None => Err(anyhow!("unknown run_id for work conn: {}", nw.run_id)),
        }
    }

    pub fn cfg(&self) -> &ServerConfig {
        &self.cfg
    }

    pub fn metrics(&self) -> &Arc<MemMetrics> {
        &self.metrics
    }

    pub async fn kick_client(&self, run_id: &str) -> Result<()> {
        let control = {
            let map = self.controls.lock().await;
            map.get(run_id).cloned()
        };
        match control {
            Some(c) => {
                c.kick("kicked from dashboard").await;
                Ok(())
            }
            None => Err(anyhow!("client not online: {run_id}")),
        }
    }

    pub async fn dashboard_snapshot(&self) -> DashboardSnapshot {
        use crate::dashboard::model::{ClientInfo, ProxyInfo};
        use std::collections::BTreeMap;

        let controls = self.controls.lock().await;
        let offline = self.offline_clients.lock().await;
        let mut clients = Vec::with_capacity(controls.len() + offline.len());
        let mut proxies = Vec::new();
        let mut proxy_type_count: BTreeMap<String, usize> = BTreeMap::new();
        let mut online_ids = std::collections::HashSet::new();

        for (_, ctrl) in controls.iter() {
            let proxy_count = ctrl.proxy_count().await;
            online_ids.insert(ctrl.run_id.clone());
            let mut cur_conns = 0usize;
            let mut client_proxies = Vec::new();
            for s in ctrl.proxy_summaries().await {
                *proxy_type_count.entry(s.proxy_type.clone()).or_default() += 1;
                let traffic = self.metrics.proxy_snapshot(&s.name);
                let proxy_conns = traffic.as_ref().map(|t| t.cur_conns).unwrap_or(0);
                cur_conns += proxy_conns;
                client_proxies.push(ProxyInfo {
                    name: s.name,
                    proxy_type: s.proxy_type,
                    remote_addr: s.remote_addr,
                    local_addr: s.local_addr,
                    client_id: ctrl.run_id.clone(),
                    status: s.status,
                    today_traffic_in: traffic.as_ref().map(|t| t.today_traffic_in).unwrap_or(0),
                    today_traffic_out: traffic.as_ref().map(|t| t.today_traffic_out).unwrap_or(0),
                    cur_conns: proxy_conns,
                    last_start_time: traffic
                        .as_ref()
                        .and_then(|t| format_proxy_time(t.last_start_at)),
                });
            }
            clients.push(ClientInfo {
                run_id: ctrl.run_id.clone(),
                user: ctrl.user.clone(),
                hostname: ctrl.hostname.clone(),
                os: ctrl.os.clone(),
                arch: ctrl.arch.clone(),
                client_ip: ctrl.client_ip.clone(),
                version: ctrl.version.clone(),
                proxy_count,
                cur_conns,
                connected_secs: ctrl.connected_at.elapsed().as_secs(),
                status: "online".into(),
            });
            proxies.extend(client_proxies);
        }

        for (id, rec) in offline.iter() {
            if online_ids.contains(id) {
                continue;
            }
            clients.push(ClientInfo {
                run_id: rec.run_id.clone(),
                user: rec.user.clone(),
                hostname: rec.hostname.clone(),
                os: rec.os.clone(),
                arch: rec.arch.clone(),
                client_ip: rec.client_ip.clone(),
                version: rec.version.clone(),
                proxy_count: rec.proxy_count,
                cur_conns: 0,
                connected_secs: rec.disconnected_at.elapsed().as_secs(),
                status: "offline".into(),
            });
        }

        clients.sort_by(|a, b| {
            let ao = a.status == "online";
            let bo = b.status == "online";
            bo.cmp(&ao).then_with(|| a.run_id.cmp(&b.run_id))
        });
        proxies.sort_by(|a, b| a.name.cmp(&b.name).then(a.client_id.cmp(&b.client_id)));

        let server_stats = self.metrics.server_snapshot();
        let total_clients = clients.len();

        DashboardSnapshot {
            clients,
            proxies,
            proxy_type_count,
            cur_conns: server_stats.cur_conns,
            total_client_counts: total_clients,
            total_traffic_in: server_stats.total_traffic_in,
            total_traffic_out: server_stats.total_traffic_out,
        }
    }
}

pub struct DashboardSnapshot {
    pub clients: Vec<crate::dashboard::model::ClientInfo>,
    pub proxies: Vec<crate::dashboard::model::ProxyInfo>,
    pub proxy_type_count: std::collections::BTreeMap<String, usize>,
    pub cur_conns: usize,
    pub total_client_counts: usize,
    pub total_traffic_in: u64,
    pub total_traffic_out: u64,
}

fn short_run_id() -> String {
    let hex = Uuid::new_v4().simple().to_string();
    hex[..16].to_owned()
}

fn valid_ident(value: &str, max_len: usize) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= max_len
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn validate_login_fields(login: &Login) -> Result<(), String> {
    if !login.run_id.trim().is_empty() && !valid_ident(&login.run_id, 64) {
        return Err("invalid run_id".into());
    }
    if login.user.len() > 64 {
        return Err("user too long".into());
    }
    if login.hostname.len() > 128 || login.os.len() > 64 || login.arch.len() > 64 {
        return Err("login metadata too long".into());
    }
    if login.version.len() > 64 {
        return Err("version too long".into());
    }
    Ok(())
}

fn format_proxy_time(unix: Option<i64>) -> Option<String> {
    let ts = unix?;
    let dt = chrono::DateTime::from_timestamp(ts, 0)?;
    Some(
        dt.with_timezone(&chrono::Local)
            .format("%m-%d %H:%M:%S")
            .to_string(),
    )
}
