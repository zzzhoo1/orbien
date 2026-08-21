use super::stream::{boxed_stream, DynStream};
use anyhow::{anyhow, Result};
use futures::future::poll_fn;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

// yamux 0.14 panics unless max_connection_receive_window >= 256 KiB * max_num_streams.
const YAMUX_STREAM_WINDOW: usize = 256 * 1024;
/// Default maximum concurrent yamux streams per physical connection.
/// Exposed as a named constant so operators know the baseline when tuning
/// `transport.max_yamux_streams` in the config file.
pub const MAX_NUM_STREAMS: usize = 256;

fn yamux_config(max_streams: usize) -> yamux::Config {
    let mut cfg = yamux::Config::default();
    let max_streams = max_streams.max(1);
    cfg.set_max_num_streams(max_streams);
    // Window must be at least YAMUX_STREAM_WINDOW * max_streams to avoid panic.
    cfg.set_max_connection_receive_window(Some(YAMUX_STREAM_WINDOW * max_streams));
    cfg
}

fn box_yamux_stream(stream: yamux::Stream) -> DynStream {
    boxed_stream(stream.compat())
}

type OpenReply = oneshot::Sender<Result<DynStream>>;

pub struct YamuxClient {
    open_tx: mpsc::Sender<OpenReply>,
    /// Configured stream limit — kept for logging/metrics.
    max_streams: usize,
}

impl YamuxClient {
    /// Start a yamux client session over `io`.
    ///
    /// `max_streams` controls the per-connection concurrency limit
    /// (default: [`MAX_NUM_STREAMS`]).
    pub fn start(io: DynStream, max_streams: usize) -> Self {
        // Channel capacity = max_streams so that back-pressure is visible
        // rather than silently dropped.
        let cap = max_streams.max(1);
        let (open_tx, open_rx) = mpsc::channel::<OpenReply>(cap);
        tokio::spawn(drive_client(io, open_rx, max_streams));
        Self {
            open_tx,
            max_streams,
        }
    }

    pub async fn open_stream(&self) -> Result<DynStream> {
        let (tx, rx) = oneshot::channel();
        self.open_tx
            .send(tx)
            .await
            .map_err(|_| anyhow!("yamux client session closed"))?;
        rx.await
            .map_err(|_| anyhow!("yamux open_stream cancelled"))?
    }

    /// Configured maximum concurrent streams for this session.
    pub fn max_streams(&self) -> usize {
        self.max_streams
    }
}

async fn drive_client(io: DynStream, mut open_rx: mpsc::Receiver<OpenReply>, max_streams: usize) {
    let mut conn =
        yamux::Connection::new(io.compat(), yamux_config(max_streams), yamux::Mode::Client);
    let mut open_count: usize = 0;
    loop {
        tokio::select! {
            cmd = open_rx.recv() => {
                match cmd {
                    Some(reply) => {
                        open_count += 1;
                        if open_count >= max_streams {
                            tracing::warn!(
                                open_count,
                                max_streams,
                                "yamux stream limit approached — consider raising transport.max_yamux_streams"
                            );
                        }
                        let res = poll_fn(|cx| conn.poll_new_outbound(cx))
                            .await
                            .map(box_yamux_stream)
                            .map_err(|e| anyhow!("yamux open outbound: {e}"));
                        // Decrement after the stream is resolved (success or error).
                        open_count = open_count.saturating_sub(1);
                        let _ = reply.send(res);
                    }
                    None => {
                        let _ = poll_fn(|cx| conn.poll_close(cx)).await;
                        break;
                    }
                }
            }
            inbound = poll_fn(|cx| conn.poll_next_inbound(cx)) => {
                match inbound {
                    Some(Ok(_stream)) => {
                        tracing::debug!("yamux client ignored unexpected inbound stream");
                    }
                    Some(Err(e)) => {
                        tracing::debug!(error = %e, "yamux client session error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}

pub async fn serve_yamux_session(
    io: DynStream,
    max_streams: usize,
    mut on_stream: impl FnMut(DynStream),
) -> Result<()> {
    let mut conn =
        yamux::Connection::new(io.compat(), yamux_config(max_streams), yamux::Mode::Server);
    loop {
        match poll_fn(|cx| conn.poll_next_inbound(cx)).await {
            Some(Ok(stream)) => {
                on_stream(box_yamux_stream(stream));
            }
            Some(Err(e)) => {
                return Err(anyhow!("yamux server accept: {e}"));
            }
            None => return Ok(()),
        }
    }
}

#[allow(dead_code)]
pub fn keepalive_duration(secs: i64) -> Duration {
    Duration::from_secs(secs.max(1) as u64)
}

pub fn client_session(io: DynStream, max_streams: usize) -> YamuxClient {
    YamuxClient::start(io, max_streams)
}
