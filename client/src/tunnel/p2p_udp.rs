//! P2P UDP relay: forward datagrams between a hole-punched peer socket and a
//! local UDP backend.
//!
//! # Data flow
//!
//! ```text
//!  remote peer  ──UDP──►  peer_sock  ──►  [run_p2p_udp_session]  ──►  backend_addr
//!  remote peer  ◄─UDP──   peer_sock  ◄──  [run_p2p_udp_session]  ◄──  backend_addr
//! ```
//!
//! The session runs until:
//! - the cancellation token is cancelled, OR
//! - either leg returns an unrecoverable I/O error.

use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

/// Maximum datagram size we will read from either leg.
const MAX_DGRAM: usize = 65_507;

/// Run a P2P UDP relay session.
///
/// Relays UDP datagrams between the connected peer socket and a local backend.
///
/// # Parameters
/// - `peer_sock`    – UDP socket already connected to the remote peer (e.g.
///                   the result of UDP hole-punching). Datagrams are read from
///                   and written back to this socket.
/// - `backend_addr` – Local service address. Every inbound datagram is
///                   forwarded here; replies are relayed back to the peer.
/// - `label`        – Short identifier used in trace spans (e.g. tunnel name).
/// - `cancel`       – Token that, when cancelled, causes the session to exit
///                   cleanly.
///
/// # Errors
/// Returns `Err` if either UDP socket encounters a fatal I/O error.
pub async fn run_p2p_udp_session(
    peer_sock: UdpSocket,
    backend_addr: SocketAddr,
    label: &str,
    cancel: CancellationToken,
) -> Result<()> {
    // Bind an ephemeral socket for the backend leg and connect it so that
    // recv() only returns packets from backend_addr.
    let backend_sock = UdpSocket::bind("127.0.0.1:0")
        .await
        .map_err(|e| anyhow!("p2p backend bind: {e}"))?;
    backend_sock
        .connect(backend_addr)
        .await
        .map_err(|e| anyhow!("p2p backend connect {backend_addr}: {e}"))?;

    let peer = Arc::new(peer_sock);
    let backend = Arc::new(backend_sock);

    tracing::debug!(label, %backend_addr, "p2p udp relay started");

    let result = relay_loop(Arc::clone(&peer), Arc::clone(&backend), cancel).await;

    tracing::debug!(label, %backend_addr, "p2p udp relay ended");
    result
}

/// Core relay loop: two concurrent copy legs under a select.
async fn relay_loop(
    peer: Arc<UdpSocket>,
    backend: Arc<UdpSocket>,
    cancel: CancellationToken,
) -> Result<()> {
    let peer_to_backend = {
        let peer = Arc::clone(&peer);
        let backend = Arc::clone(&backend);
        tokio::spawn(async move { copy_leg(&peer, &backend, "peer→backend").await })
    };
    let backend_to_peer = {
        let peer = Arc::clone(&peer);
        let backend = Arc::clone(&backend);
        tokio::spawn(async move { copy_leg(&backend, &peer, "backend→peer").await })
    };

    let result = tokio::select! {
        _ = cancel.cancelled() => Ok(()),
        r = peer_to_backend => flatten(r, "peer→backend task"),
        r = backend_to_peer  => flatten(r, "backend→peer task"),
    };

    peer.to_owned();
    backend.to_owned();
    result
}

/// Read datagrams from `src` and write them to `dst` until an error occurs.
async fn copy_leg(src: &UdpSocket, dst: &UdpSocket, tag: &str) -> Result<()> {
    let mut buf = vec![0u8; MAX_DGRAM];
    loop {
        let n = src
            .recv(&mut buf)
            .await
            .map_err(|e| anyhow!("{tag} recv: {e}"))?;
        dst.send(&buf[..n])
            .await
            .map_err(|e| anyhow!("{tag} send: {e}"))?;
    }
}

#[inline]
fn flatten<E: std::fmt::Display>(
    r: Result<Result<()>, tokio::task::JoinError>,
    tag: &str,
) -> Result<()> {
    match r {
        Ok(inner) => inner,
        Err(e) if e.is_cancelled() => Ok(()),
        Err(e) => Err(anyhow!("{tag} panicked: {e}")),
    }
}
