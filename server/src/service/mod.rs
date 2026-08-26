mod dashboard_view;
mod ingress;
mod session_registry;

use crate::access::AccessPolicy;
use crate::control::Control;
use crate::metrics::MemMetrics;
use crate::tunnel::{run_http_gw_listener, run_https_gw_listener, HttpGw, HttpsGw};
use anyhow::{anyhow, Result};
use orbien_core::config::ServerConfig;
use orbien_core::transport;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinSet;

#[allow(unused_imports)] // public API re-export
pub use dashboard_view::DashboardSnapshot;

struct OfflineClientRecord {
    session_id: String,
    user: String,
    hostname: String,
    os: String,
    arch: String,
    client_ip: String,
    version: String,
    tunnel_count: usize,
    disconnected_at: Instant,
}

pub struct Service {
    cfg: ServerConfig,
    access: Arc<AccessPolicy>,
    controls: Arc<Mutex<HashMap<String, Arc<Control>>>>,
    offline_clients: Arc<Mutex<HashMap<String, OfflineClientRecord>>>,
    http_gw: Option<Arc<HttpGw>>,
    https_gw: Option<Arc<HttpsGw>>,
    tls_config: Arc<rustls::ServerConfig>,
    metrics: Arc<MemMetrics>,
}

impl Service {
    pub fn new(cfg: ServerConfig) -> Result<Self> {
        let access = Arc::new(AccessPolicy::from_server_config(&cfg)?);
        let http_gw = if cfg.http_gw_enabled() {
            Some(Arc::new(HttpGw::new(cfg.http_gw_port)))
        } else {
            None
        };
        let https_gw = if cfg.https_gw_enabled() {
            Some(Arc::new(HttpsGw::new(cfg.https_gw_port)))
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
            http_gw,
            https_gw,
            tls_config,
            metrics: MemMetrics::new(),
        })
    }

    pub async fn run(self) -> Result<()> {
        let this = Arc::new(self);

        let tcp_addr = this.cfg.listen.clone();
        let tcp_listener = TcpListener::bind(&tcp_addr).await?;
        tracing::info!(
            %tcp_addr,
            ws_path = transport::ORBIEN_WEBSOCKET_PATH,
            tcp_mux = this.cfg.transport.tcp_mux,
            "tcp/websocket control/data listener ready"
        );

        let gw_shutdown = Arc::new(Notify::new());
        let mut set = JoinSet::new();
        let listen_host = this
            .cfg
            .listen_host()
            .map_err(|e| anyhow!("invalid listen: {e}"))?;

        if let Some(ref gw) = this.http_gw {
            let bind = this.cfg.proxy_addr.clone();
            let port = this.cfg.http_gw_port;
            let gw = Arc::clone(gw);
            let access = Arc::clone(&this.access);
            let shutdown = Arc::clone(&gw_shutdown);
            set.spawn(async move { run_http_gw_listener(bind, port, gw, access, shutdown).await });
        }

        if let Some(ref gw) = this.https_gw {
            let bind = this.cfg.proxy_addr.clone();
            let port = this.cfg.https_gw_port;
            let gw = Arc::clone(gw);
            let access = Arc::clone(&this.access);
            let shutdown = Arc::clone(&gw_shutdown);
            set.spawn(async move { run_https_gw_listener(bind, port, gw, access, shutdown).await });
        }

        if this.cfg.quic_enabled() {
            let quic_addr: SocketAddr = format!("{}:{}", listen_host, this.cfg.quic_port)
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
            tracing::info!(%quic_addr, "quic control/data listener ready");
            let svc = Arc::clone(&this);
            set.spawn(async move { svc.run_quic(endpoint).await });
        }

        if this.cfg.kcp_enabled() {
            let kcp_addr: SocketAddr = format!("{}:{}", listen_host, this.cfg.kcp_port)
                .parse()
                .map_err(|e| anyhow!("invalid kcp bind addr: {e}"))?;
            let listener = transport::bind_kcp_listener(kcp_addr).await?;
            tracing::info!(
                %kcp_addr,
                tcp_mux = this.cfg.transport.tcp_mux,
                "kcp control/data listener ready"
            );
            let svc = Arc::clone(&this);
            set.spawn(async move { svc.run_kcp(listener).await });
        }

        if this.cfg.dashboard.enabled() {
            let web_cfg = this.cfg.dashboard.clone();
            let svc = Arc::clone(&this);
            set.spawn(async move { crate::dashboard::run(svc, web_cfg).await });
        }

        let svc = Arc::clone(&this);
        set.spawn(async move { svc.run_tcp(tcp_listener).await });

        let first = set
            .join_next()
            .await
            .ok_or_else(|| anyhow!("no listener tasks"))?;
        gw_shutdown.notify_waiters();
        set.abort_all();
        while set.join_next().await.is_some() {}

        match first {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(e) if e.is_cancelled() => Ok(()),
            Err(e) => Err(anyhow!("listener task join: {e}")),
        }
    }

    pub fn cfg(&self) -> &ServerConfig {
        &self.cfg
    }

    pub fn metrics(&self) -> &Arc<MemMetrics> {
        &self.metrics
    }

    pub async fn kick_client(&self, session_id: &str) -> Result<()> {
        let control = {
            let mut map = self.controls.lock().await;
            map.remove(session_id)
        };
        match control {
            Some(c) => {
                let tunnel_count = c.tunnel_count().await;
                {
                    let mut offline = self.offline_clients.lock().await;
                    offline.insert(
                        session_id.to_string(),
                        OfflineClientRecord {
                            session_id: session_id.to_string(),
                            user: c.user.clone(),
                            hostname: c.hostname.clone(),
                            os: c.os.clone(),
                            arch: c.arch.clone(),
                            client_ip: c.client_ip.clone(),
                            version: c.version.clone(),
                            tunnel_count,
                            disconnected_at: Instant::now(),
                        },
                    );
                }
                c.kick("kicked from dashboard").await;
                Ok(())
            }
            None => Err(anyhow!("client not online: {session_id}")),
        }
    }

    /// Remove and stop a running proxy/tunnel by name (from the dashboard).
    /// Kept under the `proxy` name for backward compatibility with the
    /// dashboard API (`DELETE /api/v1/proxies/{name}`); it operates on the
    /// tunnel registry.
    pub async fn kick_proxy(&self, proxy_name: &str) -> Result<()> {
        let controls: Vec<Arc<Control>> = {
            let map = self.controls.lock().await;
            map.values().cloned().collect()
        };
        for ctrl in controls {
            if ctrl.kick_tunnel(proxy_name).await {
                return Ok(());
            }
        }
        Err(anyhow!("proxy not found: {proxy_name}"))
    }
}
