//! End-to-end integration tests for [`run_p2p_udp_session`].
//!
//! These tests use real UDP sockets on 127.0.0.1:0, a real KCP handshake
//! (`KcpListener` on the server side, `KcpStream::connect_with_config` inside
//! `run_p2p_udp_session` on the client side), and a real plain-UDP backend.
//! No mocks are used.
//!
//! Run with:
//!   cargo test --test p2p_udp_integration

use std::{net::SocketAddr, sync::Arc, time::Duration};

use kcp_tokio::{KcpConfig, KcpListener, KcpStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UdpSocket,
    time::timeout,
};
use tokio_util::sync::CancellationToken;

use orbien_client::control::p2p::run_p2p_udp_session;

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Wrap `fut` in a hard deadline; panic with `msg` on expiry.
async fn with_timeout<F, T>(dur: Duration, msg: &str, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    timeout(dur, fut)
        .await
        .unwrap_or_else(|_| panic!("timed out after {dur:?}: {msg}"))
}

/// Bind a UDP socket on an available loopback port.
async fn bind_udp() -> (UdpSocket, SocketAddr) {
    let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = sock.local_addr().unwrap();
    (sock, addr)
}

/// KCP config matching production MTU.
fn kcp_cfg() -> KcpConfig {
    KcpConfig {
        mtu: 1200,
        ..KcpConfig::default()
    }
}

/// Deadline for the KCP handshake (`KcpListener::accept`).
///
/// macOS CI runners complete the UDP-loopback KCP handshake noticeably slower
/// than Linux/Windows, so a fixed 10 s deadline is flaky there.  Use a longer
/// deadline on macOS; keep the tight 10 s elsewhere so a real regression still
/// fails fast.  This only relaxes timing — the assertion semantics of the
/// tests are unchanged.
fn kcp_accept_timeout() -> Duration {
    if cfg!(target_os = "macos") {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(10)
    }
}

/// Spawn a `KcpListener` on an available loopback port.
///
/// Returns `(server_addr, stream_rx, keepalive)`.  The `stream_rx` resolves
/// with the first accepted [`KcpStream`] once the client completes the KCP
/// handshake.  `keepalive` must be kept alive for the duration of the test:
/// `KcpListener::drop` aborts its background packet-routing task, so dropping
/// the listener right after `accept()` would silently stop routing
/// client→server datagrams to the accepted stream (breaking the B2C
/// direction).  The spawned task holds the listener until `keepalive` fires.
async fn spawn_kcp_server(
) -> (
    SocketAddr,
    tokio::sync::oneshot::Receiver<KcpStream>,
    CancellationToken,
) {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut listener = KcpListener::bind(addr, kcp_cfg())
        .await
        .expect("KcpListener::bind");
    let addr = *listener.local_addr();
    let (stream_tx, stream_rx) = tokio::sync::oneshot::channel();
    let keepalive = CancellationToken::new();
    let keepalive_task = keepalive.clone();
    tokio::spawn(async move {
        let (stream, _peer) = listener
            .accept()
            .await
            .expect("KcpListener::accept");
        let _ = stream_tx.send(stream);
        // Hold the listener (and thus its routing task) alive until the
        // test signals completion.
        keepalive_task.cancelled().await;
    });
    (addr, stream_rx, keepalive)
}

/// Bind a plain UDP backend socket wrapped in `Arc` for sharing across tasks.
async fn spawn_backend() -> (Arc<UdpSocket>, SocketAddr) {
    let (sock, addr) = bind_udp().await;
    (Arc::new(sock), addr)
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 – bidirectional payload happy path
// ─────────────────────────────────────────────────────────────────────────────
//
// Topology:
//   KCP server stream  <──KCP──>  run_p2p_udp_session  <──UDP──>  backend
//
// Data written to the KCP server stream must arrive at the backend unchanged,
// and data sent by the backend must arrive back on the KCP stream unchanged.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_bidirectional_payload_success() {
    const C2B: &[u8] = b"client-to-backend-DEADBEEF";
    const B2C: &[u8] = b"backend-to-client-CAFEBABE";

    let (server_addr, server_rx, _keepalive) = spawn_kcp_server().await;
    let (backend, backend_addr) = spawn_backend().await;

    // Connect the client socket to the KCP server so the session can perform
    // KcpStream::connect_with_transport internally.
    let (client_sock, _) = bind_udp().await;
    client_sock.connect(server_addr).await.unwrap();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(client_sock, backend_addr, "integ-bidir").await
    });

    // Wait for the KCP handshake (≤ 3 s).
    let mut kcp_srv: KcpStream = with_timeout(
        kcp_accept_timeout(),
        "KCP server accept",
        server_rx,
    )
    .await
    .expect("server accept");

    // KCP server → backend
    with_timeout(Duration::from_secs(2), "kcp write C2B", kcp_srv.write_all(C2B))
        .await
        .expect("kcp write");

    let mut buf = vec![0u8; 256];
    let (n, peer) = with_timeout(
        Duration::from_secs(2),
        "backend recv C2B",
        backend.recv_from(&mut buf),
    )
    .await
    .expect("recv_from");
    assert_eq!(&buf[..n], C2B, "backend received wrong payload");

    // backend → KCP server
    backend.send_to(B2C, peer).await.expect("backend send_to");

    let n = with_timeout(Duration::from_secs(10), "kcp read B2C", async {
        kcp_srv.read(&mut buf).await.expect("kcp read")
    })
    .await;
    assert_eq!(&buf[..n], B2C, "KCP stream received wrong reply");

    // Drop both ends.  The session is UDP-backed, so `io::join` has no EOF
    // to observe and the session may stay alive until cancelled — do not
    // require it to exit.  Just make sure it hasn't panicked.
    drop(kcp_srv);
    drop(backend);
    session.abort();
    let _ = session.await;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 – datagram boundary preservation
// ─────────────────────────────────────────────────────────────────────────────
//
// Sends 3 independent ~1100-byte KCP messages.  The backend must receive
// exactly 3 datagrams with matching size and content.  A short sleep between
// sends reduces KCP coalescing.  This test surfaces any io::join or buffering
// strategy that merges or splits application-level datagrams.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_datagram_boundary_preservation() {
    const N: usize = 3;
    const SZ: usize = 1100;

    let payloads: Vec<Vec<u8>> = (0..N)
        .map(|i| vec![(0xA0u8).wrapping_add(i as u8); SZ])
        .collect();

    let (server_addr, server_rx, _keepalive) = spawn_kcp_server().await;
    let (backend, backend_addr) = spawn_backend().await;

    let (client_sock, _) = bind_udp().await;
    client_sock.connect(server_addr).await.unwrap();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(client_sock, backend_addr, "integ-boundary").await
    });

    let mut kcp_srv: KcpStream = with_timeout(
        kcp_accept_timeout(),
        "KCP accept (boundary)",
        server_rx,
    )
    .await
    .expect("server accept");

    for payload in &payloads {
        with_timeout(Duration::from_secs(2), "kcp write payload", async {
            kcp_srv.write_all(payload).await.expect("kcp write")
        })
        .await;
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let mut received: Vec<Vec<u8>> = Vec::with_capacity(N);
    let mut buf = vec![0u8; SZ * 2];
    for i in 0..N {
        let (n, _) = with_timeout(
            Duration::from_secs(2),
            &format!("backend recv datagram {i}"),
            backend.recv_from(&mut buf),
        )
        .await
        .expect("recv_from");
        received.push(buf[..n].to_vec());
    }

    assert_eq!(received.len(), N, "expected {N} datagrams at backend");
    for (i, (got, want)) in received.iter().zip(payloads.iter()).enumerate() {
        assert_eq!(
            got.len(),
            want.len(),
            "datagram {i} size mismatch: got {} want {}",
            got.len(),
            want.len()
        );
        assert_eq!(got, want, "datagram {i} content mismatch");
    }

    drop(kcp_srv);
    drop(backend);
    session.abort();
    let _ = session.await;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3 – backend disconnect causes session to end
// ─────────────────────────────────────────────────────────────────────────────
//
// After a successful KCP handshake the backend socket is dropped.  The KCP
// server side keeps sending so the session repeatedly forwards into a dead
// UDP target.  The session must resolve (Ok or Err) within a finite timeout.
//
// We assert liveness only, not Ok vs Err: on Linux ECONNREFUSED terminates
// the send path quickly; on macOS loopback sends may succeed silently.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_backend_disconnect_ends_session() {
    let (server_addr, server_rx, _keepalive) = spawn_kcp_server().await;

    // Backend exists only long enough to get its address, then is dropped.
    let (backend_sock, backend_addr) = bind_udp().await;
    drop(backend_sock);

    let (client_sock, _) = bind_udp().await;
    client_sock.connect(server_addr).await.unwrap();

    let session = tokio::spawn(async move {
        run_p2p_udp_session(client_sock, backend_addr, "integ-disconnect").await
    });

    let mut kcp_srv: KcpStream = with_timeout(
        kcp_accept_timeout(),
        "KCP accept (disconnect)",
        server_rx,
    )
    .await
    .expect("server accept");

    // Drive the KCP side so the session hits the dead backend repeatedly.
    tokio::spawn(async move {
        loop {
            if kcp_srv.write_all(b"probe").await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // Session must terminate within 5 s; result may be Ok or Err.
    let _ = with_timeout(
        Duration::from_secs(5),
        "session must end after backend disconnect",
        session,
    )
    .await
    .expect("session task panicked");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4 – CancellationToken exits the wrapping task after handshake
// ─────────────────────────────────────────────────────────────────────────────
//
// `run_p2p_udp_session` itself does not accept a CancellationToken (matching
// the production signature).  In production, `session.rs` wraps every call in
//
//   tokio::select! {
//       _ = cancel.cancelled() => {}
//       res = run_p2p_udp_session(...) => { ... }
//   }
//
// This test verifies that pattern: after the KCP handshake the task is
// cancelled and must exit within 1 s.  No Arc::strong_count assertion.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cancellation_exits_session_task() {
    let (server_addr, server_rx, _keepalive) = spawn_kcp_server().await;
    let (backend, backend_addr) = spawn_backend().await;

    let (client_sock, _) = bind_udp().await;
    client_sock.connect(server_addr).await.unwrap();

    let cancel = CancellationToken::new();
    let cancel_task = cancel.clone();

    // Mirror the production session.rs wrapper exactly.
    let session = tokio::spawn(async move {
        tokio::select! {
            _ = cancel_task.cancelled() => Ok(()),
            res = run_p2p_udp_session(client_sock, backend_addr, "integ-cancel") => res,
        }
    });

    // Wait for KCP handshake to complete before cancelling.
    let _kcp_srv: KcpStream = with_timeout(
        kcp_accept_timeout(),
        "KCP accept (cancel)",
        server_rx,
    )
    .await
    .expect("server accept");

    // Idle briefly, then fire the token.
    tokio::time::sleep(Duration::from_millis(100)).await;
    cancel.cancel();

    // Task must exit within 1 s. The spawned task returns Result<Result<(),
    // JoinError>>: the outer is the JoinHandle, the inner is the session's
    // own Result. Consume both.
    with_timeout(
        Duration::from_secs(1),
        "session task must exit after cancel",
        session,
    )
    .await
    .expect("session task panicked")
    .expect("session task returned Err");

    // Keep backend alive until this point.
    drop(backend);
}
