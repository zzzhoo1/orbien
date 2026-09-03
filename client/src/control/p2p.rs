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

// ─────────────────────────────────────────────────────────────────────
// TCP — PRODUCTION PATH
// ─────────────────────────────────────────────────────────────────────

/// Connect the P2P `TcpStream` (from hole-punch) to the local backend service
/// for `tunnel_name`, then join the two streams bidirectionally.
///
/// # Arguments
/// * `p2p_stream` — the connected TCP stream produced by hole-punching.
/// * `local_addr`  — `host:port` string of the local backend (e.g. `"127.0.0.1:8080"`).
/// * `tunnel_name` — used only for log messages.
///
/// # Errors
/// Returns an error if the local dial fails.  The join itself is
/// best-effort; EOF on either side is treated as a clean close.
pub async fn run_p2p_tcp_session(
    p2p_stream: TcpStream,
    local_addr: &str,
    tunnel_name: &str,
) -> Result<()> {
    let local = TcpStream::connect(local_addr).await.map_err(|e| {
        anyhow!(
            "P2P TCP: dial local {} for tunnel {}: {}",
            local_addr,
            tunnel_name,
            e
        )
    })?;
    orbien_core::net::enable_nodelay(&local);
    orbien_core::net::enable_nodelay(&p2p_stream);

    tracing::info!(
        tunnel = %tunnel_name,
        %local_addr,
        "P2P TCP session: joining p2p <-> local"
    );

    if let Err(e) = io::join(p2p_stream, local).await {
        tracing::debug!(tunnel = %tunnel_name, error = %e, "P2P TCP join ended");
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────
// UDP — EXPERIMENTAL, NOT PRODUCTION
//
// DO NOT call this from handle_p2p_ready until a reliability layer is added.
// See module-level doc for the rationale.
// ─────────────────────────────────────────────────────────────────────

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

    /// Verify the experimental UDP helper compiles and can be referenced;
    /// we do NOT call it in CI because it requires a live UDP peer.
    #[test]
    fn udp_experimental_fn_exists() {
        // This test just ensures the symbol exists and the module compiles.
        let _: fn(UdpSocket, SocketAddr, usize) ->
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> =
            |s, a, b| Box::pin(run_p2p_udp_session_experimental(s, a, b));
    }
}
