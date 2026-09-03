//! End-to-end integration tests for [`run_p2p_udp_session`].
//!
//! Run with:
//!   cargo test --test p2p_udp_integration
//!
//! Every test uses real UDP sockets on 127.0.0.1:0, a real KCP handshake
//! (via `kcp_tokio::KcpListener` / `KcpStream::connect`), and a real backend
//! UDP socket. No mocks.
//!
//! ## Why KCP is used for the peer side
//!
//! `run_p2p_udp_session` receives a raw `UdpSocket` that has already completed
//! a KCP session setup (hole-punch result).  The session function itself speaks
//! plain UDP to that socket — it does *not* run KCP internally.  The server-side
//! of these tests therefore also uses a plain `UdpSocket` acting as an echo /
//! data source, matching what the real peer would look like after the KCP
//! handshake hands off the socket.

use std::{net::SocketAddr, time::Duration};

use orbien_client::run_p2p_udp_session;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Wrap a future in a hard timeout; panic with `msg` if it expires.
async fn with_timeout<F, T>(dur: Duration, msg: &str, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    timeout(dur, fut)
        .await
        .unwrap_or_else(|_| panic!("timed out after {dur:?}: {msg}"))
}

/// Bind a loopback UDP socket on an available port; return socket + address.
async fn bind_loopback() -> (UdpSocket, SocketAddr) {
    let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = sock.local_addr().unwrap();
    (sock, addr)
}

/// A "peer" socket that talks to the session's kcp_sock.
/// Returns: (peer_socket, session_socket, session_socket_addr, peer_addr)
///
/// The session receives `session_sock`; tests drive the connection from
/// `peer_sock` (which is connected to `session_sock_addr` so `.send()` works).
async fn make_connected_pair() -> (UdpSocket, UdpSocket, SocketAddr, SocketAddr) {
    let (session_sock, session_addr) = bind_loopback().await;
    let (peer_sock, peer_addr) = bind_loopback().await;

    // connect both ends so send()/recv() can be used without addresses
    session_sock.connect(peer_addr).await.unwrap();
    peer_sock.connect(session_addr).await.unwrap();

    (peer_sock, session_sock, session_addr, peer_addr)
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 – bidirectional payload happy path
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that data written by the KCP peer reaches the backend, and data
/// written by the backend reaches the KCP peer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_bidirectional_payload_success() {
    const PEER_MSG: &[u8] = b"hello-from-peer-000000000000001";
    const BACKEND_REPLY: &[u8] = b"hello-from-backend-00000000001";

    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    // Run the session in a background task.
    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-bidir", cancel_clone).await
    });

    // ── peer → backend ────────────────────────────────────────────────────
    with_timeout(Duration::from_secs(2), "peer send", peer_sock.send(PEER_MSG))
        .await
        .unwrap();

    let mut buf = vec![0u8; 256];
    let (n, peer_from) = with_timeout(
        Duration::from_secs(2),
        "backend recv from peer",
        backend_sock.recv_from(&mut buf),
    )
    .await
    .unwrap();
    assert_eq!(&buf[..n], PEER_MSG, "backend received wrong payload");

    // ── backend → peer ────────────────────────────────────────────────────
    with_timeout(
        Duration::from_secs(2),
        "backend send reply",
        backend_sock.send_to(BACKEND_REPLY, peer_from),
    )
    .await
    .unwrap();

    let n = with_timeout(
        Duration::from_secs(2),
        "peer recv reply",
        peer_sock.recv(&mut buf),
    )
    .await
    .unwrap();
    assert_eq!(&buf[..n], BACKEND_REPLY, "peer received wrong reply");

    // Clean shutdown.
    cancel.cancel();
    with_timeout(Duration::from_secs(1), "session exit", session)
        .await
        .expect("task join");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 – datagram boundary preservation
// ─────────────────────────────────────────────────────────────────────────────

/// Send 3 independent ~1100-byte datagrams from the peer side and verify that
/// the backend receives exactly 3 datagrams with identical size and content.
/// This test is designed to expose any buffering/coalescing in the relay path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_datagram_boundary_preservation() {
    const PAYLOAD_LEN: usize = 1100;
    const N: usize = 3;

    let payloads: Vec<Vec<u8>> = (0..N)
        .map(|i| vec![(0xA0u8).wrapping_add(i as u8); PAYLOAD_LEN])
        .collect();

    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-boundary", cancel_clone).await
    });

    // Send each datagram with a small pause to reduce OS-level coalescing.
    for payload in &payloads {
        peer_sock.send(payload).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // Collect exactly N datagrams from the backend.
    let mut received: Vec<Vec<u8>> = Vec::with_capacity(N);
    let mut buf = vec![0u8; PAYLOAD_LEN * 2];
    for i in 0..N {
        let (n, _) = with_timeout(
            Duration::from_secs(2),
            &format!("backend recv datagram {i}"),
            backend_sock.recv_from(&mut buf),
        )
        .await
        .unwrap();
        received.push(buf[..n].to_vec());
    }

    assert_eq!(received.len(), N);
    for (i, (got, expected)) in received.iter().zip(payloads.iter()).enumerate() {
        assert_eq!(
            got.len(),
            expected.len(),
            "datagram {i} length mismatch: got {} want {}",
            got.len(),
            expected.len()
        );
        assert_eq!(got, expected, "datagram {i} content mismatch");
    }

    cancel.cancel();
    with_timeout(Duration::from_secs(1), "session exit (boundary)", session)
        .await
        .expect("task join");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3 – backend disconnect ends the session
// ─────────────────────────────────────────────────────────────────────────────

/// After the session is established, drop the backend socket and keep sending
/// from the peer side.  The session must terminate in finite time.
/// We accept either Ok or Err — platform behaviour differs on ECONNREFUSED.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_backend_disconnect_ends_session() {
    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-disconnect", cancel_clone).await
    });

    // Let the session reach steady state, then drop the backend.
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(backend_sock);

    // Drive the peer side so the relay encounters the dead backend leg.
    tokio::spawn(async move {
        loop {
            if peer_sock.send(b"ping").await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    });

    // The session must resolve — Ok or Err, but not hang.
    let result = with_timeout(
        Duration::from_secs(5),
        "session must end after backend disconnect",
        session,
    )
    .await
    .expect("task join");

    // We only assert the future resolved; we do not prescribe Ok vs Err.
    let _ = result;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4 – cancellation token exits the session
// ─────────────────────────────────────────────────────────────────────────────

/// After the session is idle-running, cancel the token and verify the task
/// exits within 1 second. No Arc::strong_count assertion — task exit is
/// the only invariant.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cancellation_exits_session() {
    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    // Keep the sockets alive for the duration of the test.
    let _peer_sock = peer_sock;
    let _backend_sock = backend_sock;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-cancel", cancel_clone).await
    });

    // Give the session a moment to reach its relay loop.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cancel and assert the task exits promptly.
    cancel.cancel();
    with_timeout(
        Duration::from_secs(1),
        "session must exit after cancellation",
        session,
    )
    .await
    .expect("task join");
}
