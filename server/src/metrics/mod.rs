mod counter;
mod date_counter;
mod hour_counter;
mod mem;
mod traits;

pub use mem::{MemMetrics, ProxyTrafficHistory, TrafficWindow};
pub use traits::ServerMetrics;

pub const RESERVE_DAYS: usize = 7;
pub const RESERVE_HOURS: usize = 24;

pub async fn join_and_record<A, B>(
    metrics: &std::sync::Arc<MemMetrics>,
    name: &str,
    proxy_type: &str,
    visitor: A,
    work: B,
) -> std::io::Result<(u64, u64)>
where
    A: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    B: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let _guard = metrics.track_connection(name, proxy_type);
    let (to_work, from_work, err) = orbien_core::io::join_counted(visitor, work).await;
    metrics.add_traffic_in(name, proxy_type, to_work);
    metrics.add_traffic_out(name, proxy_type, from_work);
    match err {
        None => Ok((to_work, from_work)),
        Some(e) if is_benign_close(&e) => Ok((to_work, from_work)),
        Some(e) => Err(e),
    }
}

fn is_benign_close(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::UnexpectedEof
    )
}
