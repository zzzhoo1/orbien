//! End-to-end integration tests for the P2P UDP relay (`run_p2p_udp_session`).
//!
//! These tests verify the relay's real datagram-forwarding behaviour using
//! actual loopback UDP sockets and a real local UDP backend. No mocks are used.
//!
//! The relay under test forwards datagrams between a connected peer socket
//! (simulating the hole-punched far end) and the backend socket. Tests cover:
//! - bidirectional payload forwarding
//! - UDP datagram boundary preservation
//! - session termination when the backend disappears
//! - clean exit on cancellation
//! - IPv6 backend address compatibility
//!
//! Run with:
//!   cargo test --test p2p_udp_integration

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

/// Create a connected UDP socket pair that simulates the hole-punched link.
///
/// Returns `(peer_sock, session_sock, session_addr, peer_addr)`.
/// The caller passes `session_sock` to `run_p2p_udp_session` and drives
/// traffic from `peer_sock`.
async fn make_connected_pair() -> (UdpSocket, UdpSocket, SocketAddr, SocketAddr) {
    let (session_sock, session_addr) = bind_loopback().await;
    let (peer_sock, peer_addr) = bind_loopback().await;

    session_sock.connect(peer_addr).await.unwrap();
    peer_sock.connect(session_addr).await.unwrap();

    (peer_sock, session_sock, session_addr, peer_addr)
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 – bidirectional payload happy path
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that datagrams sent by the peer reach the backend unchanged, and
/// that backend replies reach the peer unchanged.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_bidirectional_payload_success() {
    const PEER_MSG: &[u8] = b"hello-from-peer-000000000000001";
    const BACKEND_REPLY: &[u8] = b"hello-from-backend-00000000001";

    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

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

    cancel.cancel();
    with_timeout(Duration::from_secs(1), "session exit", session)
        .await
        .expect("task join");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 – UDP datagram boundary preservation
// ─────────────────────────────────────────────────────────────────────────────

/// Send 3 independent ~1100-byte datagrams from the peer side and verify that
/// the backend receives exactly 3 datagrams with identical size and content.
///
/// This test exposes any buffering or coalescing inside the relay path: each
/// UDP datagram must arrive at the backend as a separate, intact message.
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

/// After the relay is running, drop the backend socket and keep sending from
/// the peer side. The session must terminate in finite time.
/// We accept either Ok or Err — platform behaviour on ECONNREFUSED differs.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_backend_disconnect_ends_session() {
    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-disconnect", cancel_clone).await
    });

    // Let the relay reach steady state, then drop the backend.
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(backend_sock);

    // Keep sending from the peer so the relay encounters the dead backend leg.
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

    let _ = result;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4 – cancellation token exits the session
// ─────────────────────────────────────────────────────────────────────────────

/// After the relay is idle-running, cancel the token and verify the task
/// exits within 1 second.
///
/// The test asserts the session is still pending immediately before cancellation
/// to rule out a false pass caused by an early exit.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cancellation_exits_session() {
    let (peer_sock, session_sock, _session_addr, _peer_addr) = make_connected_pair().await;
    let (backend_sock, backend_addr) = bind_loopback().await;

    let _peer_sock = peer_sock;
    let _backend_sock = backend_sock;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-cancel", cancel_clone).await
    });

    // Give the relay a moment to reach its loop, then verify it is still running.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        !session.is_finished(),
        "session exited before cancel was fired"
    );

    // Cancel and assert the task exits promptly.
    cancel.cancel();
    let join_result = with_timeout(
        Duration::from_secs(1),
        "session must exit after cancellation",
        session,
    )
    .await
    .expect("task must not panic");

    // Cancel arm always returns Ok(()).
    join_result.expect("session must return Ok after clean cancel");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5 – IPv6 backend address compatibility
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that run_p2p_udp_session starts successfully when backend_addr is an
/// IPv6 loopback address. Sends one datagram and asserts the backend receives it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_ipv6_backend_addr() {
    const MSG: &[u8] = b"ipv6-test-payload";

    // Build an IPv6-connected peer pair.
    let session_sock = UdpSocket::bind("[::1]:0").await.unwrap();
    let session_addr = session_sock.local_addr().unwrap();
    let peer_sock = UdpSocket::bind("[::1]:0").await.unwrap();
    let peer_addr = peer_sock.local_addr().unwrap();
    session_sock.connect(peer_addr).await.unwrap();
    peer_sock.connect(session_addr).await.unwrap();

    // IPv6 backend.
    let backend_sock = UdpSocket::bind("[::1]:0").await.unwrap();
    let backend_addr = backend_sock.local_addr().unwrap();

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(session_sock, backend_addr, "test-ipv6", cancel_clone).await
    });

    with_timeout(Duration::from_secs(2), "peer send ipv6", peer_sock.send(MSG))
        .await
        .unwrap();

    let mut buf = vec![0u8; 64];
    let (n, _) = with_timeout(
        Duration::from_secs(2),
        "backend recv ipv6",
        backend_sock.recv_from(&mut buf),
    )
    .await
    .unwrap();
    assert_eq!(&buf[..n], MSG, "IPv6 backend received wrong payload");

    cancel.cancel();
    with_timeout(Duration::from_secs(1), "session exit ipv6", session)
        .await
        .expect("task join");
}
