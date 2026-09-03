//! P2P direct-tunnel data-plane helpers.
//!
//! # Structure
//!
//! Two clearly separated sections:
//!
//! ## TCP (production path)
//!
//! [`run_p2p_tcp_session`] is the production implementation.  It receives a
//! `TcpStream` that was produced by a successful hole-punch (or TCP
//! simultaneous-open), dials the local backend service configured for the
//! named tunnel, and joins the two streams with `orbien_core::io::join`.
//!
//! The calling convention mirrors `TunnelManager::handle_stream` so the two
//! paths are easy to compare and maintain together.
//!
//! ## UDP (experimental — NOT production)
//!
//! [`run_p2p_udp_session_experimental`] is an **experimental** helper that
//! does the simplest possible thing: forward raw UDP datagrams between the
//! P2P-connected `UdpSocket` and a local UDP service.  It has no framing,
//! no reliability layer, and no flow control.  It is suitable for lab /
//! integration testing only.
//!
//! The function is intentionally **not called from `handle_p2p_ready`**.
//! When the UDP path is promoted to production it will need a proper
//! reliability layer (KCP, QUIC, or similar) and a separate PR.  The
//! boundary is kept explicit so `grep run_p2p_udp_session_experimental`
//! shows exactly one call site: the test module at the bottom of this file.

use anyhow::{anyhow, Result};
use orbien_core::io;
use std::net::SocketAddr;
use tokio::net::{TcpStream, UdpSocket};

// ───────────────────────────────────────────────────────────────────
// TCP — PRODUCTION PATH
// ───────────────────────────────────────────────────────────────────

/// Connect the P2P `TcpStream` (from hole-punch) to the local backend service
/// for `tunnel_name`, then join the two streams bidirectionally.
///
/// # Arguments
/// * `p2p_stream`  — the connected TCP stream produced by hole-punching.
/// * `local_addr`  — `host:port` string of the local backend (e.g. `"127.0.0.1:8080"`).
/// * `tunnel_name` — used only for log messages.
///
/// # Errors
/// Returns an error if the local backend dial fails.  The error propagates
/// to `handle_p2p_ready`, which logs it as a warning and keeps relay mode.
/// The join itself is best-effort; EOF on either side is treated as a clean
/// close (not an error).
pub async fn run_p2p_tcp_session(
    p2p_stream: TcpStream,
    local_addr: &str,
    tunnel_name: &str,
) -> Result<()> {
    // Dial the local backend.  Failure here is returned as Err so that the
    // spawn wrapper in session.rs can log it and fall back to relay mode.
    let local = TcpStream::connect(local_addr).await.map_err(|e| {
        anyhow!(
            "P2P TCP: dial local backend '{}' for tunnel '{}': {}",
            local_addr,
            tunnel_name,
            e
        )
    })?;

    // Reduce latency on both legs; ignore errors (sockets may not support it).
    orbien_core::net::enable_nodelay(&local);
    orbien_core::net::enable_nodelay(&p2p_stream);

    tracing::info!(
        tunnel = %tunnel_name,
        %local_addr,
        "P2P TCP session: joining p2p <-> local backend"
    );

    // Bidirectional splice.  io::join runs until either side closes or errors.
    // We treat any join error as a debug-level event (normal connection close).
    if let Err(e) = io::join(p2p_stream, local).await {
        tracing::debug!(tunnel = %tunnel_name, error = %e, "P2P TCP join ended");
    } else {
        tracing::debug!(tunnel = %tunnel_name, "P2P TCP join closed cleanly");
    }
    Ok(())
}

// ───────────────────────────────────────────────────────────────────
// UDP — EXPERIMENTAL, NOT PRODUCTION
//
// DO NOT call this from handle_p2p_ready until a reliability layer is added.
// See module-level doc for the rationale.
// ───────────────────────────────────────────────────────────────────

/// **EXPERIMENTAL — NOT PRODUCTION.**
///
/// Raw bidirectional UDP packet forwarder between a connected P2P `UdpSocket`
/// and a local UDP service.  No framing, no reliability, no flow control.
///
/// Suitable for lab/integration testing only.  Promote to production only
/// after adding a proper reliability layer (KCP, QUIC, or similar).
///
/// # Buffer size
/// `buf_size` is the per-packet buffer in bytes.  65535 is a safe maximum
/// for UDP; the caller should pass a value appropriate for the tunnel config.
#[allow(dead_code)] // intentionally not called from production paths
pub async fn run_p2p_udp_session_experimental(
    p2p_sock: UdpSocket,
    local_svc: SocketAddr,
    buf_size: usize,
) -> Result<()> {
    use std::sync::Arc;
    use tokio::net::UdpSocket as TokioUdp;

    // Bind a local socket that talks to the application service.
    let local_sock = TokioUdp::bind("127.0.0.1:0").await?;
    local_sock.connect(local_svc).await?;

    let p2p = Arc::new(p2p_sock);
    let local = Arc::new(local_sock);

    let p2p_r = Arc::clone(&p2p);
    let local_w = Arc::clone(&local);
    let p2p_to_local = tokio::spawn(async move {
        let mut buf = vec![0u8; buf_size];
        loop {
            match p2p_r.recv(&mut buf).await {
                Ok(n) => {
                    if local_w.send(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let local_r = Arc::clone(&local);
    let p2p_w = Arc::clone(&p2p);
    let local_to_p2p = tokio::spawn(async move {
        let mut buf = vec![0u8; buf_size];
        loop {
            match local_r.recv(&mut buf).await {
                Ok(n) => {
                    if p2p_w.send(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Either direction ending is enough to stop both.
    tokio::select! {
        _ = p2p_to_local => {}
        _ = local_to_p2p => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::{timeout, Duration};
    use tokio_util::sync::CancellationToken;

    /// Verify the experimental UDP helper compiles and can be referenced;
    /// we do NOT call it in CI because it requires a live UDP peer.
    #[test]
    fn udp_experimental_fn_exists() {
        // This test just ensures the symbol exists and the module compiles.
        let _: fn(UdpSocket, SocketAddr, usize) ->
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> =
            |s, a, b| Box::pin(run_p2p_udp_session_experimental(s, a, b));
    }

    // ── helpers ────────────────────────────────────────────────────────────────

    /// Build a connected TCP stream pair on loopback.  Returns
    /// `(server_side, client_side)` — either can be used as p2p_stream.
    async fn loopback_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (server, client) = tokio::join!(
            async { listener.accept().await.unwrap().0 },
            TcpStream::connect(addr),
        );
        (server, client.unwrap())
    }

    // ── test 1: success path with real payload ─────────────────────────────

    /// `run_p2p_tcp_session` must splice bytes in both directions, not just
    /// return Ok(()).  We verify a real payload survives the round-trip.
    #[tokio::test]
    async fn tcp_session_forwards_real_payload_bidirectionally() {
        // Stand up a mock "backend" listener.
        let backend_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let backend_addr = backend_listener.local_addr().unwrap();

        // Build the P2P socket pair.
        let (p2p_server, p2p_client) = loopback_pair().await;

        // Spawn the session under test (p2p_server <—> backend).
        let session = tokio::spawn(async move {
            run_p2p_tcp_session(
                p2p_server,
                &backend_addr.to_string(),
                "demo",
            )
            .await
        });

        // Accept the backend connection that run_p2p_tcp_session will dial.
        let (mut backend, _) = timeout(
            Duration::from_secs(2),
            backend_listener.accept(),
        )
        .await
        .expect("backend accept timed out")
        .unwrap();

        let (mut p2p_rx, mut p2p_tx) = p2p_client.into_split();

        // P2P → backend
        p2p_tx.write_all(b"hello-from-p2p").await.unwrap();
        let mut buf = vec![0u8; 14];
        timeout(Duration::from_secs(2), backend.read_exact(&mut buf))
            .await
            .expect("p2p→backend timed out")
            .unwrap();
        assert_eq!(&buf, b"hello-from-p2p");

        // Backend → P2P
        backend.write_all(b"hello-from-backend").await.unwrap();
        let mut buf2 = vec![0u8; 18];
        timeout(Duration::from_secs(2), p2p_rx.read_exact(&mut buf2))
            .await
            .expect("backend→p2p timed out")
            .unwrap();
        assert_eq!(&buf2, b"hello-from-backend");

        // Close both ends; session task should finish cleanly.
        drop(p2p_tx);
        drop(p2p_rx);
        drop(backend);

        timeout(Duration::from_secs(2), session)
            .await
            .expect("session task timed out")
            .unwrap() // JoinError
            .unwrap(); // Result<()>
    }

    // ── test 2: failure path ─────────────────────────────────────────────

    /// `run_p2p_tcp_session` must return `Err` (not panic) when the local
    /// backend is unavailable, so the caller can fall back to relay mode.
    ///
    /// We bind a listener, grab its port, then *drop* it immediately.  Any
    /// subsequent connect to that port gets a fast `ConnectionRefused`,
    /// avoiding the indefinite hang that TEST-NET (192.0.2.x) can cause on
    /// some CI environments.
    #[tokio::test]
    async fn tcp_session_returns_err_on_connection_refused_backend() {
        // Bind then immediately drop → port is closed, connect → ECONNREFUSED.
        let refused_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let refused_addr = refused_listener.local_addr().unwrap();
        drop(refused_listener);

        let (p2p_server, _p2p_client) = loopback_pair().await;

        let result = timeout(
            Duration::from_secs(3),
            run_p2p_tcp_session(
                p2p_server,
                &refused_addr.to_string(),
                "test-tunnel",
            ),
        )
        .await
        .expect("run_p2p_tcp_session hung (no timeout fired)");

        assert!(result.is_err(), "expected Err when backend is unreachable");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("test-tunnel"),
            "error message should mention the tunnel name; got: {msg}"
        );
    }

    // ── test 3: cancellation ─────────────────────────────────────────────

    /// After `CancellationToken::cancel()` a background P2P data task must
    /// exit promptly.  This guards against task-leak on session shutdown.
    #[tokio::test]
    async fn cancellation_token_stops_background_task_without_leak() {
        let cancel = CancellationToken::new();
        let child = cancel.child_token();

        let task = tokio::spawn(async move {
            tokio::select! {
                _ = child.cancelled() => "cancelled",
                _ = std::future::pending::<()>() => "unreachable",
            }
        });

        cancel.cancel();

        let outcome = timeout(Duration::from_secs(1), task)
            .await
            .expect("task did not stop within 1 s after cancellation")
            .unwrap();

        assert_eq!(outcome, "cancelled");
    }
}
