use super::stream::{boxed_stream, DynStream};
use super::tls::{self, client_crypto_from_tls_files, server_crypto_from_tls_files};
use anyhow::{Context, Result};
use quinn::{
    ClientConfig, Connection, Endpoint, EndpointConfig, RecvStream, SendStream, ServerConfig,
    VarInt,
};
use std::net::{SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::watch;

const STREAM_RECV_WINDOW: u32 = 32 * 1024 * 1024;
const CONN_RECV_WINDOW: u32 = 64 * 1024 * 1024;
const SEND_WINDOW: u64 = 64 * 1024 * 1024;
const UDP_BUFFER_SIZE: usize = 8 * 1024 * 1024;

pub struct QuicBiStream {
    send: SendStream,
    recv: RecvStream,
}

impl QuicBiStream {
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self { send, recv }
    }

    pub fn boxed(self) -> DynStream {
        boxed_stream(self)
    }
}

pub fn quic_bi(send: SendStream, recv: RecvStream) -> DynStream {
    QuicBiStream::new(send, recv).boxed()
}

impl AsyncRead for QuicBiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.recv).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuicBiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match Pin::new(&mut self.send).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => Poll::Ready(Ok(n)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send).poll_shutdown(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A long-lived QUIC client session.
///
/// A background task monitors `conn.closed()` so that `open_stream` fails
/// fast instead of blocking indefinitely after the underlying connection drops.
pub struct QuicSession {
    conn: Connection,
    /// Sender is dropped when the background monitor task detects closure.
    _closed_tx: watch::Sender<bool>,
    closed_rx: watch::Receiver<bool>,
    _endpoint: Endpoint,
}

impl QuicSession {
    #[allow(clippy::too_many_arguments)] // 8 config params; kept flat for direct callers
    pub async fn dial(
        server_addr: SocketAddr,
        server_name: &str,
        keepalive: Duration,
        idle_timeout: Duration,
        max_incoming_streams: u32,
        tls_cert: &str,
        tls_key: &str,
        tls_ca: &str,
    ) -> Result<Self> {
        let endpoint = build_client_endpoint(
            keepalive,
            idle_timeout,
            Some(max_incoming_streams),
            tls_cert,
            tls_key,
            tls_ca,
        )?;
        let conn = endpoint
            .connect(server_addr, server_name)
            .context("quic connect")?
            .await
            .context("quic handshake")?;
        tracing::info!(%server_addr, "quic session established");

        let (closed_tx, closed_rx) = watch::channel(false);
        // Background monitor: signals closed_tx when the connection ends.
        let monitor_conn = conn.clone();
        let monitor_tx = closed_tx.clone();
        tokio::spawn(async move {
            let reason = monitor_conn.closed().await;
            tracing::warn!(%server_addr, error = %reason, "quic connection closed");
            let _ = monitor_tx.send(true);
        });

        Ok(Self {
            conn,
            _closed_tx: closed_tx,
            closed_rx,
            _endpoint: endpoint,
        })
    }

    /// Open a new bidirectional stream, failing immediately if the connection
    /// has already been detected as closed by the monitor task.
    pub async fn open_stream(&self) -> Result<DynStream> {
        if *self.closed_rx.borrow() {
            return Err(anyhow::anyhow!("quic connection already closed"));
        }
        let (send, recv) = self.conn.open_bi().await.context("quic open_bi")?;
        Ok(QuicBiStream::new(send, recv).boxed())
    }

    /// Returns `true` if the monitor task has detected that the underlying
    /// connection is closed.  Useful for `select!` loops.
    pub fn is_closed(&self) -> bool {
        *self.closed_rx.borrow()
    }

    pub fn remote_address(&self) -> SocketAddr {
        self.conn.remote_address()
    }
}

fn high_perf_transport(
    keepalive: Duration,
    idle_timeout: Duration,
    max_streams: Option<u32>,
) -> Result<quinn::TransportConfig> {
    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(idle_timeout.try_into().context("idle timeout")?));
    transport.keep_alive_interval(Some(keepalive));

    transport.stream_receive_window(VarInt::from_u32(STREAM_RECV_WINDOW));
    transport.receive_window(VarInt::from_u32(CONN_RECV_WINDOW));
    transport.send_window(SEND_WINDOW);

    transport.initial_rtt(Duration::from_millis(5));
    if let Some(n) = max_streams {
        transport.max_concurrent_bidi_streams(n.into());
    }
    Ok(transport)
}

fn tune_udp_buffers(socket: &UdpSocket) {
    let sock = socket2::SockRef::from(socket);
    let _ = sock.set_recv_buffer_size(UDP_BUFFER_SIZE);
    let _ = sock.set_send_buffer_size(UDP_BUFFER_SIZE);
    if let Ok(sz) = sock.recv_buffer_size() {
        tracing::info!(recv_buf = sz, "quic udp SO_RCVBUF");
    }
    if let Ok(sz) = sock.send_buffer_size() {
        tracing::info!(send_buf = sz, "quic udp SO_SNDBUF");
    }
}

fn runtime() -> Result<Arc<dyn quinn::Runtime>> {
    quinn::default_runtime().context("no quinn async runtime (tokio)")
}

pub fn build_server_endpoint(
    bind: SocketAddr,
    keepalive: Duration,
    idle_timeout: Duration,
    max_streams: u32,
    tls_cert: &str,
    tls_key: &str,
    tls_ca: &str,
) -> Result<Endpoint> {
    let crypto = server_crypto_from_tls_files(tls_cert, tls_key, tls_ca)?;
    let mut server = ServerConfig::with_crypto(Arc::new(crypto));
    server.transport_config(Arc::new(high_perf_transport(
        keepalive,
        idle_timeout,
        Some(max_streams),
    )?));

    let socket = UdpSocket::bind(bind).context("bind quic udp")?;
    tune_udp_buffers(&socket);
    Endpoint::new(EndpointConfig::default(), Some(server), socket, runtime()?)
        .context("quic Endpoint::new server")
}

pub fn build_client_endpoint(
    keepalive: Duration,
    idle_timeout: Duration,
    max_streams: Option<u32>,
    tls_cert: &str,
    tls_key: &str,
    tls_ca: &str,
) -> Result<Endpoint> {
    let _ = tls::install_ring_provider();
    let crypto = client_crypto_from_tls_files(tls_cert, tls_key, tls_ca)?;
    let mut client = ClientConfig::new(Arc::new(crypto));
    client.transport_config(Arc::new(high_perf_transport(
        keepalive,
        idle_timeout,
        max_streams,
    )?));

    let socket = UdpSocket::bind("0.0.0.0:0").context("bind quic client udp")?;
    tune_udp_buffers(&socket);
    let mut endpoint = Endpoint::new(EndpointConfig::default(), None, socket, runtime()?)
        .context("quic Endpoint::new client")?;
    endpoint.set_default_client_config(client);
    Ok(endpoint)
}
