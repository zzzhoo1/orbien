mod data_pool;
mod register;

use crate::access::AccessPolicy;
use crate::metrics::{MemMetrics, ServerMetrics};
use crate::tunnel::{HttpGw, HttpsGw, TunnelManager};
use anyhow::Result;
use orbien_core::config::ServerConfig;
use orbien_core::msg::{self, KickOut, Message, Ping, Pong};
use orbien_core::transport::DynStream;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::task::JoinSet;
use tokio::time::sleep;

type CtrlRead = ReadHalf<DynStream>;
type CtrlWrite = WriteHalf<DynStream>;

pub struct Control {
    pub session_id: String,
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
    data_tx: mpsc::Sender<DynStream>,
    data_rx: Mutex<mpsc::Receiver<DynStream>>,
    data_notify: Notify,
    shutdown_notify: Notify,
    tunnels: Mutex<TunnelManager>,
    bg_tasks: Mutex<JoinSet<()>>,
    closed: AtomicBool,
    pool_count: usize,
    http_gw: Option<Arc<HttpGw>>,
    https_gw: Option<Arc<HttpsGw>>,
    access: Arc<AccessPolicy>,
    pub metrics: Arc<MemMetrics>,
    last_ping_unix: AtomicI64,
}

impl Control {
    #[allow(clippy::too_many_arguments)] // Control::new wires many subsystem handles
    pub fn new(
        session_id: String,
        stream: DynStream,
        cfg: ServerConfig,
        pool_count: usize,
        http_gw: Option<Arc<HttpGw>>,
        https_gw: Option<Arc<HttpsGw>>,
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
        let (data_tx, data_rx) = mpsc::channel(64);
        Self {
            session_id,
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
            data_tx,
            data_rx: Mutex::new(data_rx),
            data_notify: Notify::new(),
            shutdown_notify: Notify::new(),
            tunnels: Mutex::new(TunnelManager::new()),
            bg_tasks: Mutex::new(JoinSet::new()),
            closed: AtomicBool::new(false),
            pool_count: pool_count.max(1),
            http_gw,
            https_gw,
            access,
            metrics,
            last_ping_unix: AtomicI64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            ),
        }
    }

    pub async fn tunnel_summaries(&self) -> Vec<crate::tunnel::TunnelSummary> {
        self.tunnels.lock().await.summaries()
    }

    pub async fn tunnel_count(&self) -> usize {
        self.tunnels.lock().await.len()
    }

    /// Remove and stop a single registered tunnel by name (from the dashboard).
    pub async fn kick_tunnel(&self, name: &str) -> bool {
        let mut tm = self.tunnels.lock().await;
        if let Some(ty) = tm.remove(name).await {
            self.metrics.close_tunnel(name, ty);
            tracing::info!(
                session_id = %self.session_id,
                tunnel = %name,
                "tunnel kicked from dashboard"
            );
            true
        } else {
            false
        }
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        for _ in 0..self.pool_count {
            if self.closed.load(Ordering::SeqCst) {
                return Ok(());
            }
            self.request_data_conn().await?;
        }

        {
            let timeout = self.effective_ping_timeout();
            if timeout > 0 {
                let this = Arc::clone(&self);
                self.bg_tasks.lock().await.spawn(async move {
                    loop {
                        if this.closed.load(Ordering::SeqCst) {
                            break;
                        }
                        sleep(Duration::from_secs(1)).await;
                        let last = this.last_ping_unix.load(Ordering::Relaxed);
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        if last > 0 && now.saturating_sub(last) > timeout {
                            tracing::warn!(
                                session_id = %this.session_id,
                                timeout_secs = timeout,
                                "heartbeat timeout"
                            );
                            this.shutdown().await;
                            break;
                        }
                    }
                });
            }
        }

        loop {
            if self.closed.load(Ordering::SeqCst) {
                break;
            }
            let msg = tokio::select! {
                _ = self.shutdown_notify.notified() => {
                    break;
                }
                msg = async {
                    let mut reader = self.reader.lock().await;
                    msg::read_msg(&mut *reader).await
                } => {
                    match msg {
                        Ok(m) => m,
                        Err(e) => {
                            if !self.closed.load(Ordering::SeqCst) {
                                tracing::debug!(error = %e, "control read ended");
                            }
                            break;
                        }
                    }
                }
            };

            match msg {
                Message::NewTunnel(np) => self.handle_new_tunnel(np).await?,
                Message::CloseTunnel(cp) => self.handle_close_tunnel(cp).await?,
                Message::Ping(p) => self.handle_ping(p).await?,
                other => {
                    tracing::warn!(ty = other.type_byte(), "ignored control message");
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            self.shutdown_notify.notify_waiters();
            self.data_notify.notify_waiters();
            return;
        }
        self.shutdown_notify.notify_waiters();
        self.data_notify.notify_waiters();
        {
            let mut tm = self.tunnels.lock().await;
            for (name, ty) in tm.close_all().await {
                self.metrics.close_tunnel(&name, ty);
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
        tracing::info!(session_id = %self.session_id, %reason, "kicking client");
        self.shutdown().await;
    }

    fn effective_ping_timeout(&self) -> i64 {
        let hb_to = self.cfg.transport.heartbeat_timeout;
        if hb_to > 0 {
            return hb_to;
        }
        if self.cfg.transport.tcp_mux {
            let mux_ka = self.cfg.transport.mux_keepalive_secs;
            if mux_ka > 0 {
                return mux_ka.saturating_mul(3);
            }
        }
        -1
    }

    async fn handle_ping(&self, _p: Ping) -> Result<()> {
        self.last_ping_unix.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        let mut writer = self.writer.lock().await;
        msg::write_msg(&mut *writer, &Message::Pong(Pong::default())).await?;
        Ok(())
    }
}
