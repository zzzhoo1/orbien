use crate::access::AccessPolicy;
use crate::metrics::{MemMetrics, ServerMetrics};
use crate::proxy::{
    format_local_addr, HttpProxy, HttpVhost, HttpsProxy, HttpsVhost, ProxyManager, RegisteredProxy,
    TcpProxy, UdpProxy,
};
use anyhow::{anyhow, Result};
use orbien_core::config::ServerConfig;
use orbien_core::msg::{
    self, CloseProxy, KickOut, LoginResp, Message, NewProxy, NewProxyResp, Ping, Pong,
    ReqWorkConn, StartWorkConn,
};
use orbien_core::transport::DynStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::task::JoinSet;
use tokio::time::sleep;

type CtrlRead = ReadHalf<DynStream>;
type CtrlWrite = WriteHalf<DynStream>;

pub struct Control {
    pub run_id: String,
    pub user: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub version: String,
    pub client_ip: String,
    pub connected_at: Instant,
    cfg: ServerConfig,
    reader: Mutex<CtrlRead>,
    writer: Mutex<CtrlWrite>,
    work_tx: mpsc::Sender<DynStream>,
    work_rx: Mutex<mpsc::Receiver<DynStream>>,
    work_notify: Notify,
    proxies: Mutex<ProxyManager>,
    bg_tasks: Mutex<JoinSet<()>>,
    closed: AtomicBool,
    pool_count: usize,
    http_vhost: Option<Arc<HttpVhost>>,
    https_vhost: Option<Arc<HttpsVhost>>,
    access: Arc<AccessPolicy>,
    pub metrics: Arc<MemMetrics>,
    /// Timestamp of the last received control message.
    last_seen: StdMutex<Instant>,
}

impl Control {
    pub fn new(
        run_id: String,
        stream: DynStream,
        cfg: ServerConfig,
        pool_count: usize,
        http_vhost: Option<Arc<HttpVhost>>,
        https_vhost: Option<Arc<HttpsVhost>>,
        access: Arc<AccessPolicy>,
        user: String,
        hostname: String,
        os: String,
        arch: String,
        version: String,
        client_ip: String,
        metrics: Arc<MemMetrics>,
    ) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        let (work_tx, work_rx) = mpsc::channel(64);
        Self {
            run_id,
            user,
            hostname,
            os,
            arch,
            version,
            client_ip,
            connected_at: Instant::now(),
            cfg,
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            work_tx,
            work_rx: Mutex::new(work_rx),
            work_notify: Notify::new(),
            proxies: Mutex::new(ProxyManager::new()),
            bg_tasks: Mutex::new(JoinSet::new()),
            closed: AtomicBool::new(false),
            pool_count: pool_count.max(1),
            http_vhost,
            https_vhost,
            access,
            metrics,
            last_seen: StdMutex::new(Instant::now()),
        }
    }

    pub async fn send_login_resp(&self, response: LoginResp) -> Result<()> {
        let mut writer = self.writer.lock().await;
        msg::write_msg(&mut *writer, &Message::LoginResp(response)).await
    }

    pub async fn proxy_summaries(&self) -> Vec<crate::proxy::ProxySummary> {
        self.proxies.lock().await.summaries()
    }

    pub async fn proxy_count(&self) -> usize {
        self.proxies.lock().await.len()
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        for _ in 0..self.pool_count {
            self.request_work_conn().await?;
        }

        // ── Heartbeat background task (configurable interval / timeout) ─────
        {
            let ctl = Arc::clone(&self);
            let interval = ctl.cfg.ctrl_heartbeat_interval();
            let timeout = ctl.cfg.ctrl_heartbeat_timeout();
            self.bg_tasks.lock().await.spawn(async move {
                loop {
                    sleep(interval).await;

                    if ctl.closed.load(Ordering::SeqCst) {
                        break;
                    }

                    let since = {
                        let ls = *ctl.last_seen.lock().unwrap();
                        Instant::now().saturating_duration_since(ls)
                    };
                    if since >= timeout {
                        tracing::warn!(
                            run_id = %ctl.run_id,
                            elapsed_secs = since.as_secs(),
                            "control heartbeat timeout, closing connection"
                        );
                        ctl.shutdown().await;
                        break;
                    }

                    let mut writer = ctl.writer.lock().await;
                    if let Err(e) =
                        msg::write_msg(&mut *writer, &Message::Ping(Ping::default())).await
                    {
                        tracing::warn!(
                            run_id = %ctl.run_id,
                            error = %e,
                            "control ping write error, closing connection"
                        );
                        drop(writer);
                        ctl.shutdown().await;
                        break;
                    }
                    tracing::trace!(run_id = %ctl.run_id, "control ping sent");
                }
            });
        }

        loop {
            if self.closed.load(Ordering::SeqCst) {
                break;
            }
            let msg = {
                let mut reader = self.reader.lock().await;
                msg::read_msg(&mut *reader).await?
            };

            // Every received message resets the liveness timer.
            *self.last_seen.lock().unwrap() = Instant::now();

            match msg {
                Message::NewProxy(np) => self.handle_new_proxy(np).await?,
                Message::CloseProxy(cp) => self.handle_close_proxy(cp).await?,
                Message::Ping(p) => self.handle_ping(p).await?,
                Message::Pong(_) => {
                    tracing::trace!(run_id = %self.run_id, "control pong received");
                }
                other => {
                    tracing::warn!(ty = other.type_byte(), "ignored control message");
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.work_notify.notify_waiters();
        {
            let mut pm = self.proxies.lock().await;
            for (name, ty) in pm.close_all().await {
                self.metrics.close_proxy(&name, ty);
            }
        }
        {
            let mut writer = self.writer.lock().await;
            let _ = writer.shutdown().await;
        }
        let mut bg = self.bg_tasks.lock().await;
        bg.abort_all();
        while bg.join_next().await.is_some() {}
    }

    pub async fn kick(&self, reason: impl Into<String>) {
        let reason = reason.into();
        {
            let mut writer = self.writer.lock().await;
            let _ = msg::write_msg(
                &mut *writer,
                &Message::KickOut(KickOut {
                    reason: reason.clone(),
                }),
            )
            .await;
        }
        tracing::info!(run_id = %self.run_id, %reason, "kicking client");
        self.shutdown().await;
    }

    fn note_proxy_registered(&self, name: &str, proxy_type: &str) {
        self.metrics
            .new_proxy(name, proxy_type, &self.user, &self.run_id);
    }

    pub async fn push_work_conn(&self, stream: DynStream) {
        let _ = self.work_tx.send(stream).await;
        self.work_notify.notify_waiters();
    }

    async fn try_pop_work(&self) -> Option<DynStream> {
        let mut rx = self.work_rx.lock().await;
        rx.try_recv().ok()
    }

    async fn spawn_refill(self: &Arc<Self>) {
        let ctl = Arc::clone(self);
        self.bg_tasks.lock().await.spawn(async move {
            if ctl.closed.load(Ordering::SeqCst) {
                return;
            }
            let _ = ctl.request_work_conn().await;
        });
    }

    pub async fn get_work_conn(self: &Arc<Self>) -> Result<DynStream> {
        if let Some(conn) = self.try_pop_work().await {
            self.spawn_refill().await;
            return Ok(conn);
        }

        self.request_work_conn().await?;

        // Use configurable timeout from ServerConfig
        let timeout = self.cfg.work_conn_timeout();
        let deadline = Instant::now() + timeout;
        loop {
            if self.closed.load(Ordering::SeqCst) {
                return Err(anyhow!("control closed while waiting for work conn"));
            }
            if let Some(conn) = self.try_pop_work().await {
                self.spawn_refill().await;
                return Ok(conn);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(anyhow!(
                    "timeout waiting for work conn ({}s)",
                    timeout.as_secs()
                ));
            }
            tokio::select! {
                _ = self.work_notify.notified() => {}
                _ = sleep(remaining.min(Duration::from_millis(100))) => {}
            }
        }
    }

    async fn request_work_conn(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(anyhow!("control closed"));
        }
        let mut writer = self.writer.lock().await;
        msg::write_msg(&mut *writer, &Message::ReqWorkConn(ReqWorkConn {})).await?;
        Ok(())
    }

    async fn handle_new_proxy(self: &Arc<Self>, np: NewProxy) -> Result<()> {
        let resp = match self.register_proxy(&np).await {
            Ok(remote_addr) => NewProxyResp {
                proxy_name: np.proxy_name.clone(),
                remote_addr,
                error: String::new(),
            },
            Err(e) => NewProxyResp {
                proxy_name: np.proxy_name.clone(),
                remote_addr: String::new(),
                error: e.to_string(),
            },
        };

        let mut writer = self.writer.lock().await;
        msg::write_msg(&mut *writer, &Message::NewProxyResp(resp)).await?;
        Ok(())
    }

    fn validate_proxy_name(name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(anyhow!("proxy name is required"));
        }
        if name.len() > 128 {
            return Err(anyhow!("proxy name too long"));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(anyhow!("proxy name contains invalid characters"));
        }
        Ok(())
    }

    async fn register_proxy(self: &Arc<Self>, np: &NewProxy) -> Result<String> {
        Self::validate_proxy_name(&np.proxy_name)?;
        match np.proxy_type.as_str() {
            "tcp" => self.register_tcp_proxy(np).await,
            "http" => self.register_http_proxy(np).await,
            "https" => self.register_https_proxy(np).await,
            "udp" => self.register_udp_proxy(np).await,
            other => Err(anyhow!("unsupported proxy type: {other}")),
        }
    }

    async fn register_tcp_proxy(self: &Arc<Self>, np: &NewProxy) -> Result<String> {
        if np.remote_port <= 0 || np.remote_port > 65535 {
            return Err(anyhow!("invalid remote_port"));
        }

        let limiter = orbien_core::limit::limiter_if_mode(
            &np.bandwidth_limit,
            &np.bandwidth_limit_mode,
            orbien_core::limit::BandwidthLimitMode::Server,
        )?;
        if let Some(ref l) = limiter {
            tracing::info!(
                proxy = %np.proxy_name,
                bytes_per_sec = l.bytes_per_sec(),
                mode = "server",
                "bandwidth limit enabled"
            );
        }

        let bind_addr = self.cfg.proxy_bind_addr.clone();
        let remote_port = np.remote_port as u16;
        let name = np.proxy_name.clone();
        let control = Arc::clone(self);
        let max_connections = np.max_connections;

        if max_connections > 0 {
            tracing::info!(proxy = %name, max_connections, "connection limit configured");
        }

        {
            let mut pm = self.proxies.lock().await;
            if let Some(old_ty) = pm.remove(&name).await {
                self.metrics.close_proxy(&name, old_ty);
            }
        }

        let proxy = TcpProxy::start(
            name.clone(),
            bind_addr,
            remote_port,
            control,
            limiter,
            Arc::clone(&self.access),
            max_connections,
        )
        .await?;
        let remote_addr = format!(":{}", remote_port);

        let local_addr = format_local_addr(&np.local_ip, np.local_port);
        let mut pm = self.proxies.lock().await;
        pm.insert(name.clone(), RegisteredProxy::Tcp(proxy), local_addr)
            .await;
        self.note_proxy_registered(&name, "tcp");
        tracing::info!(proxy = %np.proxy_name, port = remote_port, "tcp proxy registered");
        Ok(remote_addr)
    }

    async fn register_http_proxy(self: &Arc<Self>, np: &NewProxy) -> Result<String> {
        let vhost = self
            .http_vhost
            .clone()
            .ok_or_else(|| anyhow!("http proxy requires server vhostHTTPPort > 0"))?;

        let limiter = orbien_core::limit::limiter_if_mode(
            &np.bandwidth_limit,
            &np.bandwidth_limit_mode,
            orbien_core::limit::BandwidthLimitMode::Server,
        )?;
        if let Some(ref l) = limiter {
            tracing::info!(
                proxy = %np.proxy_name,
                bytes_per_sec = l.bytes_per_sec(),
                mode = "server",
                "bandwidth limit enabled"
            );
        }

        let name = np.proxy_name.clone();
        if np.max_connections > 0 {
            tracing::info!(proxy = %name, max_connections = np.max_connections, "connection limit configured");
        }
        {
            let mut pm = self.proxies.lock().await;
            if let Some(old_ty) = pm.remove(&name).await {
                self.metrics.close_proxy(&name, old_ty);
            }
        }

        let proxy = HttpProxy::register(
            np,
            Arc::clone(self),
            Arc::clone(&vhost),
            &self.cfg.sub_domain_host,
            limiter,
        )
        .await?;

        let remote_addr = proxy
            .domains
            .iter()
            .map(|d| format!("{d}:{}", vhost.listen_port))
            .collect::<Vec<_>>()
            .join(",");

        let local_addr = format_local_addr(&np.local_ip, np.local_port);
        let mut pm = self.proxies.lock().await;
        pm.insert(name.clone(), RegisteredProxy::Http(proxy), local_addr)
            .await;
        self.note_proxy_registered(&name, "http");
        Ok(remote_addr)
    }

    async fn register_https_proxy(self: &Arc<Self>, np: &NewProxy) -> Result<String> {
        let vhost = self
            .https_vhost
            .clone()
            .ok_or_else(|| anyhow!("https proxy requires server vhostHTTPSPort > 0"))?;

        let limiter = orbien_core::limit::limiter_if_mode(
            &np.bandwidth_limit,
            &np.bandwidth_limit_mode,
            orbien_core::limit::BandwidthLimitMode::Server,
        )?;
        if let Some(ref l) = limiter {
            tracing::info!(
                proxy = %np.proxy_name,
                bytes_per_sec = l.bytes_per_sec(),
                mode = "server",
                "bandwidth limit enabled"
            );
        }

        let name = np.proxy_name.clone();
        if np.max_connections > 0 {
            tracing::info!(proxy = %name, max_connections = np.max_connections, "connection limit configured");
        }
        {
            let mut pm = self.proxies.lock().await;
            if let Some(old_ty) = pm.remove(&name).await {
                self.metrics.close_proxy(&name, old_ty);
            }
        }

        let proxy = HttpsProxy::register(
            np,
            Arc::clone(self),
            Arc::clone(&vhost),
            &self.cfg.sub_domain_host,
            limiter,
        )
        .await?;

        let remote_addr = proxy
            .domains
            .iter()
            .map(|d| format!("{d}:{}", vhost.listen_port))
            .collect::<Vec<_>>()
            .join(",");

        let local_addr = format_local_addr(&np.local_ip, np.local_port);
        let mut pm = self.proxies.lock().await;
        pm.insert(name.clone(), RegisteredProxy::Https(proxy), local_addr)
            .await;
        self.note_proxy_registered(&name, "https");
        Ok(remote_addr)
    }

    async fn register_udp_proxy(self: &Arc<Self>, np: &NewProxy) -> Result<String> {
        if np.remote_port <= 0 || np.remote_port > 65535 {
            return Err(anyhow!("invalid remote_port"));
        }

        let limiter = orbien_core::limit::limiter_if_mode(
            &np.bandwidth_limit,
            &np.bandwidth_limit_mode,
            orbien_core::limit::BandwidthLimitMode::Server,
        )?;
        if let Some(ref l) = limiter {
            tracing::info!(
                proxy = %np.proxy_name,
                bytes_per_sec = l.bytes_per_sec(),
                mode = "server",
                "bandwidth limit enabled"
            );
        }

        let bind_addr = self.cfg.proxy_bind_addr.clone();
        let remote_port = np.remote_port as u16;
        let name = np.proxy_name.clone();
        let control = Arc::clone(self);
        let packet_size = self.cfg.udp_packet_size.max(512);

        {
            let mut pm = self.proxies.lock().await;
            if let Some(old_ty) = pm.remove(&name).await {
                self.metrics.close_proxy(&name, old_ty);
            }
        }

        let max_connections = np.max_connections;
        if max_connections > 0 {
            tracing::info!(proxy = %name, max_connections, "udp session limit configured");
        }

        let proxy = UdpProxy::start(
            name.clone(),
            bind_addr,
            remote_port,
            control,
            limiter,
            packet_size,
            max_connections,
        )
        .await?;
        let remote_addr = format!(":{}", remote_port);

        let local_addr = format_local_addr(&np.local_ip, np.local_port);
        let mut pm = self.proxies.lock().await;
        pm.insert(name.clone(), RegisteredProxy::Udp(proxy), local_addr)
            .await;
        self.note_proxy_registered(&name, "udp");
        tracing::info!(proxy = %np.proxy_name, port = remote_port, "udp proxy registered");
        Ok(remote_addr)
    }

    async fn handle_close_proxy(&self, cp: CloseProxy) -> Result<()> {
        let mut pm = self.proxies.lock().await;
        if let Some(ty) = pm.remove(&cp.proxy_name).await {
            self.metrics.close_proxy(&cp.proxy_name, ty);
        }
        Ok(())
    }

    async fn handle_ping(&self, _p: Ping) -> Result<()> {
        let mut writer = self.writer.lock().await;
        msg::write_msg(&mut *writer, &Message::Pong(Pong::default())).await?;
        Ok(())
    }

    pub async fn start_work_conn(
        &self,
        mut work: DynStream,
        proxy_name: &str,
        src_addr: String,
        src_port: u16,
        dst_addr: String,
        dst_port: u16,
    ) -> Result<DynStream> {
        msg::write_msg(
            &mut work,
            &Message::StartWorkConn(StartWorkConn {
                proxy_name: proxy_name.to_string(),
                src_addr,
                src_port,
                dst_addr,
                dst_port,
                error: String::new(),
            }),
        )
        .await?;
        Ok(work)
    }
}
