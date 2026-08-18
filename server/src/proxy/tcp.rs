use super::{acquire_conn, ConnGuard};
use crate::access::{prepare_visitor, AccessPolicy};
use crate::control::Control;
use crate::metrics;
use anyhow::Result;
use orbien_core::limit::{maybe_limit, BandwidthLimiter};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

pub struct TcpProxy {
    pub name: String,
    pub remote_port: u16,
    /// Current number of active (in-flight) connections.
    pub active_conns: Arc<AtomicUsize>,
    /// Optional upper bound on simultaneous connections (0 = unlimited).
    pub max_connections: usize,
    closed: Arc<AtomicBool>,
    notify: Arc<Notify>,
    accept_task: Mutex<Option<JoinHandle<()>>>,
}

impl TcpProxy {
    pub async fn start(
        name: String,
        bind_addr: String,
        remote_port: u16,
        control: Arc<Control>,
        limiter: Option<Arc<BandwidthLimiter>>,
        access: Arc<AccessPolicy>,
        max_connections: usize,
    ) -> Result<Self> {
        let addr = format!("{bind_addr}:{remote_port}");
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!(%addr, proxy = %name, max_connections, "tcp proxy listening");

        let closed = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());
        let active_conns = Arc::new(AtomicUsize::new(0));

        let closed_flag = Arc::clone(&closed);
        let notify_wait = Arc::clone(&notify);
        let active_conns_spawn = Arc::clone(&active_conns);
        let proxy_name = name.clone();
        let limiter_spawn = limiter.clone();
        let control_weak = Arc::downgrade(&control);

        let accept_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify_wait.notified() => break,
                    accepted = listener.accept() => {
                        match accepted {
                            Ok((user_conn, peer)) => {
                                if closed_flag.load(Ordering::SeqCst) { break; }
                                let Some(ctl) = control_weak.upgrade() else { break; };

                                if !acquire_conn(&active_conns_spawn, max_connections) {
                                    tracing::warn!(
                                        proxy = %proxy_name,
                                        max_connections,
                                        peer = %peer,
                                        "connection limit reached, dropping"
                                    );
                                    drop(user_conn);
                                    continue;
                                }

                                let pname = proxy_name.clone();
                                let lim = limiter_spawn.clone();
                                let access = Arc::clone(&access);
                                let active = Arc::clone(&active_conns_spawn);

                                tokio::spawn(async move {
                                    // #6 — Access log
                                    tracing::info!(
                                        proxy = %pname,
                                        peer = %peer,
                                        "tcp connection accepted"
                                    );
                                    match handle_user_conn(
                                        ctl, &pname, user_conn, peer, lim, access, active,
                                    ).await {
                                        Ok(()) => tracing::info!(
                                            proxy = %pname,
                                            peer = %peer,
                                            "tcp connection closed"
                                        ),
                                        Err(e) => tracing::info!(
                                            proxy = %pname,
                                            peer = %peer,
                                            error = %e,
                                            "tcp connection closed with error"
                                        ),
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "accept failed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            name,
            remote_port,
            active_conns,
            max_connections,
            closed,
            notify,
            accept_task: Mutex::new(Some(accept_task)),
        })
    }

    /// Returns the current number of active connections.
    pub fn active_connections(&self) -> usize {
        self.active_conns.load(Ordering::Relaxed)
    }

    pub async fn close(&self) {
        tracing::info!(
            proxy = %self.name,
            remote_port = self.remote_port,
            "tcp proxy closing"
        );
        self.closed.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
        if let Some(h) = self.accept_task.lock().await.take() {
            h.abort();
            let _ = h.await;
        }
    }
}

impl Drop for TcpProxy {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
        if let Some(h) = self.accept_task.get_mut().take() {
            h.abort();
        }
    }
}

async fn handle_user_conn(
    control: Arc<Control>,
    proxy_name: &str,
    user_conn: tokio::net::TcpStream,
    peer: std::net::SocketAddr,
    limiter: Option<Arc<BandwidthLimiter>>,
    access: Arc<AccessPolicy>,
    active_conns: Arc<AtomicUsize>,
) -> Result<()> {
    let _guard = ConnGuard(Arc::clone(&active_conns));

    let visitor = prepare_visitor(user_conn, peer, &access).await?;
    let work = control.get_work_conn().await?;
    let work = control
        .start_work_conn(
            work,
            proxy_name,
            visitor.visitor.ip().to_string(),
            visitor.visitor.port(),
            visitor.local.map(|a| a.ip().to_string()).unwrap_or_default(),
            visitor.local.map(|a| a.port()).unwrap_or(0),
        )
        .await?;

    let work = maybe_limit(work, limiter);

    tracing::debug!(
        proxy = %proxy_name,
        peer = %visitor.peer,
        visitor = %visitor.visitor,
        "joining visitor <-> work"
    );
    let _ = metrics::join_and_record(
        &control.metrics, proxy_name, "tcp", visitor.stream, work,
    ).await;
    Ok(())
}
