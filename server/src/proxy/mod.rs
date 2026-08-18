mod http;
mod https;
mod manager;
mod tcp;
mod udp;
mod vhost;

pub use http::{run_vhost_http_listener, HttpProxy};
pub use https::{run_vhost_https_listener, HttpsProxy, HttpsVhost};
pub use manager::{format_local_addr, ProxyManager, ProxySummary, RegisteredProxy};
pub use tcp::TcpProxy;
pub use udp::UdpProxy;
pub use vhost::HttpVhost;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub(crate) struct ConnGuard(pub Arc<AtomicUsize>);

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) fn acquire_conn(active: &Arc<AtomicUsize>, max_connections: usize) -> bool {
    if max_connections > 0 {
        let current = active.fetch_add(1, Ordering::SeqCst);
        if current >= max_connections {
            active.fetch_sub(1, Ordering::SeqCst);
            return false;
        }
    } else {
        active.fetch_add(1, Ordering::Relaxed);
    }
    true
}
