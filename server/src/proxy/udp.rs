use crate::control::Control;
use crate::metrics::{MemMetrics, ServerMetrics};
use anyhow::Result;
use orbien_core::limit::{maybe_limit, BandwidthLimiter};
use orbien_core::msg::{self, Message, UdpPacket};
use orbien_core::udp::{forward_user_conn, CHANNEL_CAP};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

pub struct UdpProxy {
    pub name: String,
    pub remote_port: u16,
    closed: Arc<AtomicBool>,
    notify: Arc<Notify>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    _udp: Arc<UdpSocket>,
}

impl UdpProxy {
    pub async fn start(
        name: String,
        bind_addr: String,
        remote_port: u16,
        control: Arc<Control>,
        limiter: Option<Arc<BandwidthLimiter>>,
        packet_size: usize,
        max_connections: usize,
    ) -> Result<Self> {
        let addr = format!("{bind_addr}:{remote_port}");
        let udp = Arc::new(UdpSocket::bind(&addr).await?);
        tracing::info!(%addr, proxy = %name, max_connections, "udp proxy listening");

        let closed = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());

        let (send_tx, send_rx) = mpsc::channel::<UdpPacket>(CHANNEL_CAP);
        let (read_tx, read_rx) = mpsc::channel::<UdpPacket>(CHANNEL_CAP);

        // Read the configurable deadline once; avoid cloning cfg repeatedly.
        let work_read_deadline = control.cfg().udp_work_read_deadline();

        let mut tasks = Vec::new();

        {
            let udp_f = Arc::clone(&udp);
            let notify_f = Arc::clone(&notify);
            tasks.push(tokio::spawn(async move {
                tokio::select! {
                    _ = notify_f.notified() => {}
                    _ = forward_user_conn(udp_f, read_rx, send_tx, packet_size, max_connections) => {}
                }
            }));
        }

        {
            let closed_flag = Arc::clone(&closed);
            let notify_wait = Arc::clone(&notify);
            let proxy_name = name.clone();
            let control = Arc::downgrade(&control);
            tasks.push(tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                work_conn_loop(
                    proxy_name,
                    control,
                    limiter,
                    send_rx,
                    read_tx,
                    closed_flag,
                    notify_wait,
                    work_read_deadline,
                )
                .await;
            }));
        }

        Ok(Self {
            name,
            remote_port,
            closed,
            notify,
            tasks: Mutex::new(tasks),
            _udp: udp,
        })
    }

    pub async fn close(&self) {
        tracing::info!(proxy = %self.name, remote_port = self.remote_port, "udp proxy closing");
        self.closed.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
        let tasks = std::mem::take(&mut *self.tasks.lock().await);
        for h in tasks {
            h.abort();
            let _ = h.await;
        }
    }
}

impl Drop for UdpProxy {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
        for h in self.tasks.get_mut().drain(..) {
            h.abort();
        }
    }
}

async fn abort_wait(h: JoinHandle<()>) {
    h.abort();
    let _ = h.await;
}

#[allow(clippy::too_many_arguments)]
async fn work_conn_loop(
    proxy_name: String,
    control: std::sync::Weak<Control>,
    limiter: Option<Arc<BandwidthLimiter>>,
    mut send_rx: mpsc::Receiver<UdpPacket>,
    read_tx: mpsc::Sender<UdpPacket>,
    closed: Arc<AtomicBool>,
    notify: Arc<Notify>,
    work_read_deadline: Duration,
) {
    while !closed.load(Ordering::SeqCst) {
        let Some(control) = control.upgrade() else {
            return;
        };

        let work = {
            tokio::select! {
                _ = notify.notified() => return,
                w = control.get_work_conn() => w,
            }
        };

        let work = match work {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!(proxy = %proxy_name, error = %e, "udp get work conn failed");
                tokio::select! {
                    _ = notify.notified() => return,
                    _ = sleep(Duration::from_secs(1)) => continue,
                }
            }
        };

        let work = match control
            .start_work_conn(work, &proxy_name, String::new(), 0, String::new(), 0)
            .await
        {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!(proxy = %proxy_name, error = %e, "udp StartWorkConn failed");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let work = maybe_limit(work, limiter.clone());
        let (reader, mut writer) = tokio::io::split(work);
        tracing::info!(proxy = %proxy_name, "udp work conn established");
        let metrics = Arc::clone(&control.metrics);
        let _guard = metrics.track_connection(&proxy_name, "udp");

        let (fail_tx, mut fail_rx) = mpsc::channel::<()>(1);
        let read_tx_r = read_tx.clone();
        let fail_r = fail_tx.clone();
        let name_r = proxy_name.clone();
        let metrics_r = Arc::clone(&metrics);
        let deadline = work_read_deadline;
        let mut reader_task = Some(tokio::spawn(async move {
            work_reader(reader, read_tx_r, fail_r, name_r, metrics_r, deadline).await;
        }));

        let reconnect = loop {
            tokio::select! {
                _ = notify.notified() => {
                    if let Some(h) = reader_task.take() { abort_wait(h).await; }
                    return;
                }
                _ = fail_rx.recv() => {
                    if let Some(h) = reader_task.take() { abort_wait(h).await; }
                    break true;
                }
                pkt = send_rx.recv() => {
                    match pkt {
                        Some(pkt) => {
                            let nbytes = pkt.content.len() as u64;
                            tracing::trace!(proxy = %proxy_name, len = nbytes, "udp packet to work");
                            if msg::write_msg(&mut writer, &Message::UdpPacket(pkt)).await.is_err() {
                                tracing::warn!(proxy = %proxy_name, "udp work write error");
                                if let Some(h) = reader_task.take() { abort_wait(h).await; }
                                break true;
                            }
                            metrics.add_traffic_in(&proxy_name, "udp", nbytes);
                        }
                        None => {
                            if let Some(h) = reader_task.take() { abort_wait(h).await; }
                            return;
                        }
                    }
                }
            }
        };

        if reconnect {
            tracing::info!(proxy = %proxy_name, "udp work conn lost; reconnecting");
        }
    }
}

async fn work_reader<R: AsyncRead + Unpin + Send + 'static>(
    mut reader: R,
    read_tx: mpsc::Sender<UdpPacket>,
    fail_tx: mpsc::Sender<()>,
    proxy_name: String,
    metrics: Arc<MemMetrics>,
    deadline: Duration,
) {
    loop {
        match timeout(deadline, msg::read_msg(&mut reader)).await {
            Ok(Ok(Message::Ping(_))) => {
                tracing::trace!(proxy = %proxy_name, "udp work ping");
            }
            Ok(Ok(Message::UdpPacket(pkt))) => {
                let nbytes = pkt.content.len() as u64;
                tracing::trace!(proxy = %proxy_name, len = nbytes, "udp packet from work");
                metrics.add_traffic_out(&proxy_name, "udp", nbytes);
                if read_tx.send(pkt).await.is_err() {
                    break;
                }
            }
            Ok(Ok(other)) => {
                tracing::warn!(proxy = %proxy_name, ty = other.type_byte(), "udp unexpected msg");
            }
            Ok(Err(e)) => {
                tracing::warn!(proxy = %proxy_name, error = %e, "udp work read error");
                let _ = fail_tx.send(()).await;
                break;
            }
            Err(_) => {
                tracing::warn!(
                    proxy = %proxy_name,
                    deadline_secs = deadline.as_secs(),
                    "udp work read deadline exceeded"
                );
                let _ = fail_tx.send(()).await;
                break;
            }
        }
    }
}
