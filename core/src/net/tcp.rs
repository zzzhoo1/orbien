use tokio::net::TcpStream;

#[inline]
pub fn enable_nodelay(stream: &TcpStream) {
    if let Err(e) = stream.set_nodelay(true) {
        tracing::trace!(error = %e, "tcp set_nodelay failed");
    }
}
