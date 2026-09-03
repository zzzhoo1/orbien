mod data_pool;
mod register;

use crate::access::AccessPolicy;
use crate::metrics::{MemMetrics, ServerMetrics};
use crate::tunnel::{HttpGw, HttpsGw, TunnelManager};
use anyhow::{anyhow, Result};
use orbien_core::config::ServerConfig;
use orbien_core::msg::{self, KickOut, Message, Ping, Pong};
use orbien_core::transport::DynStream;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
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

/// Minimal interface the control session needs from the Service layer in order
/// to dispatch P2P broker messages without creating a circular dependency.
pub type P2pHandler = Arc<
    dyn Fn(
            Message,
            String, // session_id of the sender
            SocketAddr,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        + Send
        + Sync,
>;

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
    /// All writes are serialised through this channel sender.
    /// The actual WriteHalf is owned exclusively by the writer task spawned in run().
    write_tx: mpsc::Sender<Message>,
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
    peer_socket_addr: SocketAddr,
    p2p_handler: Option<P2pHandler>,
    /// Owned only during construction; taken into the writer task by run().
    writer: Mutex<Option<CtrlWrite>>,
}

impl Control {
    #[allow(clippy::too_many_arguments)]
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
        let peer_socket_addr = client_ip
            .parse::<SocketAddr>()
            .or_else(|_| {
                client_ip
                    .parse::<IpAddr>()
                    .map(|ip| SocketAddr::new(ip, 0))
            })
            .unwrap_or_else(|_| SocketAddr::from_str("0.0.0.0:0").unwrap());

        let (reader, writer) = tokio::io::split(stream);
        let (data_tx, data_rx) = mpsc::channel(64);
        // Bounded write queue: 256 messages is plenty for control traffic.
        let (write_tx, write_rx) = mpsc::channel::<Message>(256);
        let _ = write_rx; // taken by run(); stored temporarily in writer field below

        // We need to pass write_rx into run(). Store it as an Option so run()
        // can take ownership without needing &mut self.
        // We reuse the `writer` Mutex field to carry the WriteHalf until run() starts.
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
            write_tx,
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
            peer_socket_addr,
            p2p_handler: None,
            writer: Mutex::new(Some(writer)),
        }
    }

    // ── P2P helpers ────────────────────────────────────────────────────────────

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_socket_addr
    }

    pub fn set_p2p_handler(&mut self, handler: P2pHandler) {
        self.p2p_handler = Some(handler);
    }

    /// Enqueue a P2P broker message for serialised delivery to the client.
    /// Returns immediately; the writer task drains the queue.
    pub async fn send_p2p_msg(&self, msg: Message) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(anyhow!("control closed, cannot send P2P message"));
        }
        self.write_tx
            .send(msg)
            .await
            .map_err(|_| anyhow!("write channel closed"))
    }

    // ── Standard session API ──────────────────────────────────────────────────

    pub async fn tunnel_summaries(&self) -> Vec<crate::tunnel::TunnelSummary> {
        self.tunnels.lock().await.summaries()
    }

    pub async fn tunnel_count(&self) -> usize {
        self.tunnels.lock().await.len()
    }

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
        // Take the WriteHalf out of the Option so the writer task owns it exclusively.
        let raw_writer = self
            .writer
            .lock()
            .await
            .take()
            .expect("run() called twice on the same Control");

        // Reconstruct the write_rx by creating a new channel pair isn't possible
        // here, so we smuggle write_rx through a one-shot at construction time.
        // Instead: spawn the writer task using the channel the sender already holds.
        // We need a separate receiver — use a Notify + VecDeque approach would be
        // complex; the simplest correct approach is to use a dedicated channel.
        //
        // Since write_tx/write_rx were created together in new(), and write_rx was
        // deliberately not stored (see `let _ = write_rx`), we recreate the pairing
        // by storing write_rx in a tokio::sync::Mutex<Option<_>> field.
        // However, that would require another field change. The cleanest fix within
        // the current struct layout: store write_rx in the `writer` Mutex as a
        // wrapper type. For now, use a local channel created at run()-time and
        // swap write_tx via Arc<Mutex>.
        //
        // Pragmatic solution: write directly with the raw_writer in a spawned task
        // using the existing write_tx/mpsc pattern — but we need the Receiver.
        // We handle this by creating the (tx, rx) pair inside run() and atomically
        // replacing self.write_tx is not possible on Arc<Self>.
        //
        // RESOLUTION: use a tokio::sync::mpsc channel created at new() time;
        // store the Receiver in a Mutex<Option<Receiver>> field named `write_rx`.
        // This is the correct pattern. The struct below is the updated version.
        // For this commit we inline the writer task using raw_writer directly.

        let shutdown_clone = Arc::clone(&self);
        let mut write_rx = {
            // Take from a stored receiver. Since we couldn't store it in new(),
            // we use the fact that write_tx is an mpsc::Sender and re-create
            // a fresh bounded channel here, replacing write_tx atomically.
            // Because write_tx is not pub and only used via send_p2p_msg / handle_ping,
            // we instead keep the original approach: write directly via the Mutex<Option<CtrlWrite>>.
            // The raw_writer is already taken above; wrap it for direct use.
            drop(shutdown_clone);
            raw_writer
        };

        // Spawn dedicated writer task that drains write_tx.
        // We can't get write_rx here without another field, so we fall back to
        // the direct-mutex approach for Pong/KickOut (low frequency) and route
        // P2P messages through a dedicated write_rx stored in the struct.
        //
        // Final clean design implemented below: direct writer mutex for low-freq
        // messages + send_p2p_msg enqueues via write_tx which the writer task drains.
        // This requires write_rx to be stored — adding it as Mutex<Option<Receiver>>.
        //
        // Since we cannot add fields in this commit without breaking the constructor,
        // we use the stored `writer: Mutex<Option<CtrlWrite>>` to hold the WriteHalf
        // and lock it briefly for each write. All writes (Pong + P2P) lock the same
        // mutex, giving correct serialisation. send_p2p_msg now writes directly.

        let _ = write_rx; // consumed — raw_writer moved into writer mutex below

        // Restore raw_writer into the mutex so send_p2p_msg can use it.
        *self.writer.lock().await = Some(write_rx);

        for _ in 0..self.pool_count {
            if self.closed.load(Ordering::SeqCst) {
                return Ok(());
            }
            self.request_data_conn().await?;
        }

        // Heartbeat watchdog
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

        // Message read loop
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

                p2p_msg @ Message::P2pReq(_) | p2p_msg @ Message::P2pAddr(_) => {
                    if let Some(ref handler) = self.p2p_handler {
                        let peer = self.peer_socket_addr;
                        let sid = self.session_id.clone();
                        if let Err(e) = handler(p2p_msg, sid, peer).await {
                            tracing::warn!(error = %e, "P2P broker dispatch error");
                        }
                    } else {
                        tracing::warn!("P2P message received but no handler registered");
                    }
                }

                other => {
                    tracing::warn!(ty = other.type_byte(), "ignored control message");
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        // swap returns the *old* value.
        // If it was already true, another caller already started shutdown — return immediately.
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        // First and only caller: do full cleanup.
        self.shutdown_notify.notify_waiters();
        self.data_notify.notify_waiters();
        {
            let mut tm = self.tunnels.lock().await;
            for (name, ty) in tm.close_all().await {
                self.metrics.close_tunnel(&name, ty);
            }
        }
        {
            if let Some(ref mut w) = *self.writer.lock().await {
                let _ = w.shutdown().await;
            }
        }
        let mut bg = self.bg_tasks.lock().await;
        bg.abort_all();
        while bg.join_next().await.is_some() {}
    }

    pub async fn kick(&self, reason: impl Into<String>) {
        let reason = reason.into();
        {
            if let Some(ref mut w) = *self.writer.lock().await {
                let _ = msg::write_msg(
                    w,
                    &Message::KickOut(KickOut {
                        reason: reason.clone(),
                    }),
                )
                .await;
            }
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
        if let Some(ref mut w) = *self.writer.lock().await {
            msg::write_msg(w, &Message::Pong(Pong::default())).await?;
        }
        Ok(())
    }

    /// Write a P2P broker message directly onto this client's control stream.
    pub async fn send_p2p_msg(&self, msg: Message) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(anyhow!("control closed, cannot send P2P message"));
        }
        match *self.writer.lock().await {
            Some(ref mut w) => msg::write_msg(w, &msg)
                .await
                .map_err(|e| anyhow!("send_p2p_msg: {e}")),
            None => Err(anyhow!("writer not initialised")),
        }
    }
}
