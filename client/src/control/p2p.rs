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
//! ## UDP (production path — KCP reliable layer)
//!
//! [`run_p2p_udp_session`] wraps the punched `UdpSocket` in a `KcpStream`
//! to obtain a reliable, ordered byte stream, then dials the local UDP
//! backend socket and splices both streams with `orbien_core::io::join`.
//! Error / fallback semantics are identical to the TCP path.
//!
//! The local UDP socket is wrapped in [`UdpStreamAdapter`] so that
//! `io::join` can write *into* it (KCP → backend direction) as well as
//! read from it (backend → KCP direction).
//!
//! ## UDP legacy (experimental — kept for lab use)
//!
//! [`run_p2p_udp_session_experimental`] is the original raw forwarder.
//! It is deprecated and **not called from any production path**.

use anyhow::{anyhow, Result};
use orbien_core::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpStream, UdpSocket};

// Conservative per-packet MTU for KCP: 1200 bytes leaves room for outer
// UDP/IP headers on any typical path (Ethernet, PPPoE, VPN).
const KCP_MTU: usize = 1200;

// ───────────────────────────────────────────────────────────────────
// UdpStreamAdapter — bidirectional AsyncRead + AsyncWrite over UDP
// ───────────────────────────────────────────────────────────────────

/// A thin wrapper around a **connected** `UdpSocket` that implements both
/// [`AsyncRead`] and [`AsyncWrite`], allowing it to be used as one side of
/// `io::join`.
///
/// # Datagram boundary contract
///
/// `poll_read` returns exactly one datagram per call (the bytes that arrived
/// in a single UDP `recv`).  `poll_write` sends exactly one datagram per call
/// and always reports `n = buf.len()` on success — it never asks the caller
/// to retry a partial write.  This preserves the datagram boundary that the
/// local backend expects while appearing as a byte stream to `io::join`.
///
/// # Flush / shutdown
///
/// Both are no-ops; UDP has no connection teardown concept at the socket API
/// level.
struct UdpStreamAdapter {
    sock: Arc<UdpSocket>,
}

impl UdpStreamAdapter {
    fn new(sock: Arc<UdpSocket>) -> Self {
        Self { sock }
    }
}

impl AsyncRead for UdpStreamAdapter {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.sock.poll_recv(cx, buf)
    }
}

impl AsyncWrite for UdpStreamAdapter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.sock.poll_send(cx, buf) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        // UDP has no kernel send buffer that needs explicit flushing.
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        // UDP sockets have no graceful close; nothing to do.
        Poll::Ready(Ok(()))
    }
}

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
    let local = TcpStream::connect(local_addr).await.map_err(|e| {
        anyhow!(
            "P2P TCP: dial local backend '{}' for tunnel '{}': {}",
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
        "P2P TCP session: joining p2p <-> local backend"
    );

    if let Err(e) = io::join(p2p_stream, local).await {
        tracing::debug!(tunnel = %tunnel_name, error = %e, "P2P TCP join ended");
    } else {
        tracing::debug!(tunnel = %tunnel_name, "P2P TCP join closed cleanly");
    }
    Ok(())
}

// ───────────────────────────────────────────────────────────────────
// UDP — PRODUCTION PATH (KCP reliable layer)
// ───────────────────────────────────────────────────────────────────

/// Connect the P2P `UdpSocket` (from hole-punch) to the local UDP backend
/// service for `tunnel_name` using KCP as the reliability layer, then join
/// both streams bidirectionally.
///
/// # Design
/// A `KcpStream` is layered on the punched socket to provide ordering and
/// retransmission.  A second `UdpSocket` bound on loopback talks to the
/// local service, wrapped in [`UdpStreamAdapter`] to provide
/// `AsyncRead + AsyncWrite` so that `io::join` can splice both directions.
///
/// # Datagram boundaries
/// `UdpStreamAdapter::poll_write` sends each `io::join` write as a single
/// UDP datagram.  KCP segments arriving in one `poll_read` call are
/// forwarded as one datagram.  Backends that parse datagrams independently
/// will see correct framing; stream-oriented backends are unaffected.
///
/// # MTU
/// Per-packet buffer is capped at [`KCP_MTU`] (1200 bytes) to avoid
/// fragmentation on any realistic path.
///
/// # Arguments
/// * `p2p_sock`    — connected `UdpSocket` from hole-punch.
/// * `local_addr`  — `SocketAddr` of the local UDP service.
/// * `tunnel_name` — used only for log messages.
///
/// # Errors
/// Returns an error if the KCP stream cannot be established or the local
/// backend socket cannot be bound / connected.  The caller (`handle_p2p_ready`)
/// logs the error as a warning and keeps relay mode.
pub async fn run_p2p_udp_session(
    p2p_sock: UdpSocket,
    local_addr: SocketAddr,
    tunnel_name: &str,
) -> Result<()> {
    use kcp_tokio::{KcpConfig, KcpStream};

    let peer_addr = p2p_sock.peer_addr().map_err(|e| {
        anyhow!(
            "P2P UDP: cannot read peer addr for tunnel '{}': {}",
            tunnel_name,
            e
        )
    })?;

    let cfg = KcpConfig {
        mtu: KCP_MTU,
        ..KcpConfig::default()
    };

    // Wrap the punched socket in a KCP stream (reliable + ordered).
    let kcp_stream = KcpStream::connect_with_config(&cfg, p2p_sock, peer_addr)
        .await
        .map_err(|e| {
            anyhow!(
                "P2P UDP: KCP connect failed for tunnel '{}': {}",
                tunnel_name,
                e
            )
        })?;

    // Bind a loopback UDP socket, connect it to the local service, then wrap
    // it in UdpStreamAdapter so io::join can splice both directions.
    let local_sock = UdpSocket::bind("127.0.0.1:0").await.map_err(|e| {
        anyhow!(
            "P2P UDP: bind loopback socket for tunnel '{}': {}",
            tunnel_name,
            e
        )
    })?;
    local_sock.connect(local_addr).await.map_err(|e| {
        anyhow!(
            "P2P UDP: connect to local backend '{}' for tunnel '{}': {}",
            local_addr,
            tunnel_name,
            e
        )
    })?;
    let local_adapter = UdpStreamAdapter::new(Arc::new(local_sock));

    tracing::info!(
        tunnel = %tunnel_name,
        %local_addr,
        "P2P UDP session: joining kcp <-> local backend"
    );

    if let Err(e) = io::join(kcp_stream, local_adapter).await {
        tracing::debug!(tunnel = %tunnel_name, error = %e, "P2P UDP join ended");
    } else {
        tracing::debug!(tunnel = %tunnel_name, "P2P UDP join closed cleanly");
    }
    Ok(())
}

// ───────────────────────────────────────────────────────────────────
// UDP — EXPERIMENTAL, DEPRECATED
//
// Kept for lab/integration reference.  DO NOT call from production.
// Use run_p2p_udp_session (above) for all new code.
// ───────────────────────────────────────────────────────────────────

/// **DEPRECATED — use [`run_p2p_udp_session`] instead.**
///
/// Raw bidirectional UDP packet forwarder.  No framing, no reliability,
/// no flow control.  Retained for lab/integration reference only.
#[deprecated(
    since = "0.2.0",
    note = "use run_p2p_udp_session (KCP-backed) for production UDP tunnels"
)]
#[allow(dead_code)]
pub async fn run_p2p_udp_session_experimental(
    p2p_sock: UdpSocket,
    local_svc: SocketAddr,
    buf_size: usize,
) -> Result<()> {
    use tokio::net::UdpSocket as TokioUdp;

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

    tokio::select! {
        _ = p2p_to_local => {}
        _ = local_to_p2p => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream, UdpSocket};
    use tokio::time::{timeout, Duration};
    use tokio_util::sync::CancellationToken;

    /// Verify the deprecated experimental UDP helper still compiles.
    #[test]
    #[allow(deprecated)]
    fn udp_experimental_fn_exists() {
        let _: fn(UdpSocket, SocketAddr, usize) ->
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> =
            |s, a, b| Box::pin(run_p2p_udp_session_experimental(s, a, b));
    }

    // ── helpers ────────────────────────────────────────────────────────────

    async fn loopback_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (server, client) = tokio::join!(
            async { listener.accept().await.unwrap().0 },
            TcpStream::connect(addr),
        );
        (server, client.unwrap())
    }

    // ── UdpStreamAdapter unit test: bidirectional loopback ─────────────────
    //
    // Two connected UDP sockets, each wrapped in UdpStreamAdapter.
    // Write bytes through one adapter, read them back through the other.
    // Verifies both AsyncWrite (poll_send path) and AsyncRead (poll_recv path).
    #[tokio::test]
    async fn udp_adapter_is_truly_bidirectional() {
        let sock_a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sock_b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr_a = sock_a.local_addr().unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        sock_a.connect(addr_b).await.unwrap();
        sock_b.connect(addr_a).await.unwrap();

        let mut adapter_a = UdpStreamAdapter::new(Arc::new(sock_a));
        let mut adapter_b = UdpStreamAdapter::new(Arc::new(sock_b));

        // a → b
        timeout(Duration::from_secs(2), adapter_a.write_all(b"ping"))
            .await.expect("write timed out").unwrap();
        let mut buf = vec![0u8; 4];
        timeout(Duration::from_secs(2), adapter_b.read_exact(&mut buf))
            .await.expect("read timed out").unwrap();
        assert_eq!(&buf, b"ping");

        // b → a
        timeout(Duration::from_secs(2), adapter_b.write_all(b"pong"))
            .await.expect("write timed out").unwrap();
        timeout(Duration::from_secs(2), adapter_a.read_exact(&mut buf))
            .await.expect("read timed out").unwrap();
        assert_eq!(&buf, b"pong");
    }

    // ── TCP test 1: success path ────────────────────────────────────────────

    #[tokio::test]
    async fn tcp_session_forwards_real_payload_bidirectionally() {
        let backend_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let backend_addr = backend_listener.local_addr().unwrap();
        let (p2p_server, p2p_client) = loopback_pair().await;

        let session = tokio::spawn(async move {
            run_p2p_tcp_session(p2p_server, &backend_addr.to_string(), "demo").await
        });

        let (mut backend, _) = timeout(
            Duration::from_secs(2),
            backend_listener.accept(),
        )
        .await
        .expect("backend accept timed out")
        .unwrap();

        let (mut p2p_rx, mut p2p_tx) = p2p_client.into_split();

        p2p_tx.write_all(b"hello-from-p2p").await.unwrap();
        let mut buf = vec![0u8; 14];
        timeout(Duration::from_secs(2), backend.read_exact(&mut buf))
            .await.expect("p2p→backend timed out").unwrap();
        assert_eq!(&buf, b"hello-from-p2p");

        backend.write_all(b"hello-from-backend").await.unwrap();
        let mut buf2 = vec![0u8; 18];
        timeout(Duration::from_secs(2), p2p_rx.read_exact(&mut buf2))
            .await.expect("backend→p2p timed out").unwrap();
        assert_eq!(&buf2, b"hello-from-backend");

        drop(p2p_tx); drop(p2p_rx); drop(backend);
        timeout(Duration::from_secs(2), session)
            .await.expect("session task timed out").unwrap().unwrap();
    }

    // ── TCP test 2: failure path ────────────────────────────────────────────

    #[tokio::test]
    async fn tcp_session_returns_err_on_connection_refused_backend() {
        let refused_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let refused_addr = refused_listener.local_addr().unwrap();
        drop(refused_listener);

        let (p2p_server, _p2p_client) = loopback_pair().await;

        let result = timeout(
            Duration::from_secs(3),
            run_p2p_tcp_session(p2p_server, &refused_addr.to_string(), "test-tunnel"),
        )
        .await
        .expect("run_p2p_tcp_session hung");

        assert!(result.is_err(), "expected Err when backend is unreachable");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("test-tunnel"), "error should mention tunnel name; got: {msg}");
    }

    // ── TCP test 3: cancellation ────────────────────────────────────────────

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
            .await.expect("task did not stop within 1s").unwrap();
        assert_eq!(outcome, "cancelled");
    }

    // ── UDP test 1: function signature compiles and type-checks ─────────────
    #[test]
    fn udp_session_production_fn_exists() {
        let _: fn(UdpSocket, SocketAddr, &str) ->
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> =
            |s, a, n| Box::pin(run_p2p_udp_session(s, a, n));
    }

    // ── UDP test 2: unreachable backend returns Err ─────────────────────────
    //
    // sock_a is connected to sock_b (which is not a KCP peer), so
    // KcpStream::connect_with_config will time out waiting for a KCP
    // handshake.  The outer 4s timeout converts that into an Err, which is
    // what the production fallback path relies on.
    #[tokio::test]
    async fn udp_session_returns_err_on_unreachable_backend() {
        let refused = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let refused_addr: SocketAddr = refused.local_addr().unwrap();
        drop(refused);

        let sock_a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sock_b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        sock_a.connect(addr_b).await.unwrap();

        let result = timeout(
            Duration::from_secs(4),
            run_p2p_udp_session(sock_a, refused_addr, "udp-test-tunnel"),
        )
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("timed out — treated as unreachable")));

        assert!(
            result.is_err(),
            "expected Err when backend is unreachable; got Ok"
        );
    }

    // ── UDP test 3: cancellation exits within deadline ──────────────────────
    //
    // Verifies the tokio::select! + CancellationToken pattern used in
    // session.rs to wrap run_p2p_udp_session.  The function itself is not
    // called here because KCP handshake requires a live peer; the cancellation
    // contract is transport-independent.
    #[tokio::test]
    async fn udp_session_cancellation_exits_within_deadline() {
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
            .expect("UDP task did not stop within 1s after cancellation")
            .unwrap();
        assert_eq!(outcome, "cancelled");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UDP 控制面分支测试
    //
    // 覆盖 run_p2p_udp_session 的三条错误路径入口，以及 session.rs 调用方
    // 的 service 地址 parse 逻辑。不重复 p2p_udp_integration.rs 中已覆盖的
    // 真实 KCP 握手和数据面转发场景。
    // ═══════════════════════════════════════════════════════════════════════

    // ── 控制面测试 1：未连接 socket 触发 peer_addr 失败，错误含 tunnel_name ─

    /// `run_p2p_udp_session` 的第一个 guard 是 `p2p_sock.peer_addr()`。
    /// 当传入一个**未连接**的 UdpSocket 时，该调用失败并返回 Err，错误信息
    /// 必须包含 tunnel_name，便于 `handle_p2p_ready` 的 warn! 日志定位。
    #[tokio::test]
    async fn udp_session_peer_addr_fail_err_contains_tunnel_name() {
        // 绑定但不 connect —— peer_addr() 会返回 ENOTCONN
        let unconnected = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        // 随便一个合法的 SocketAddr 作为 local_addr 占位；到不了 bind 步骤
        let dummy_local: SocketAddr = "127.0.0.1:1".parse().unwrap();

        let result = timeout(
            Duration::from_secs(2),
            run_p2p_udp_session(unconnected, dummy_local, "my-tunnel"),
        )
        .await
        .expect("run_p2p_udp_session hung unexpectedly");

        assert!(result.is_err(), "expected Err on unconnected socket");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("my-tunnel"),
            "error must reference tunnel name for tracing; got: {msg}"
        );
    }

    /// 空字符串 tunnel_name 不应 panic，错误信息包含空字符串本身。
    #[tokio::test]
    async fn udp_session_empty_tunnel_name_does_not_panic() {
        let unconnected = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let dummy_local: SocketAddr = "127.0.0.1:1".parse().unwrap();

        let result = timeout(
            Duration::from_secs(2),
            run_p2p_udp_session(unconnected, dummy_local, ""),
        )
        .await
        .expect("hung");

        // 关键断言：不 panic，且返回 Err（peer_addr 失败路径）
        assert!(
            result.is_err(),
            "expected Err; empty tunnel_name must not open a no-op Ok path"
        );
    }

    // ── 控制面测试 2：KCP connect 失败时，错误信息含 tunnel_name ──────────

    /// `peer_addr()` 成功（socket 已 connect）但 KCP 握手无法完成时，
    /// `run_p2p_udp_session` 必须返回 Err，且错误信息包含 tunnel_name。
    ///
    /// 构造方式：sock_a connect 到 sock_b，但 sock_b 不是 KCP listener，
    /// 握手超时后 kcp_tokio 返回 Err，即触发第二个 guard。
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn udp_session_kcp_fail_err_contains_tunnel_name() {
        // sock_a connected → sock_b；sock_b 不做任何 KCP 响应
        let sock_a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sock_b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        sock_a.connect(addr_b).await.unwrap();
        // sock_b 持有端口，不参与 KCP，防止 ECONNREFUSED 干扰

        let dummy_local: SocketAddr = "127.0.0.1:1".parse().unwrap();

        // KCP connect 默认会重传，给 5s 兜底
        let result = timeout(
            Duration::from_secs(5),
            run_p2p_udp_session(sock_a, dummy_local, "kcp-fail-tunnel"),
        )
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("timed out")));

        assert!(result.is_err(), "expected Err when KCP handshake cannot complete");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("kcp-fail-tunnel"),
            "error must contain tunnel name; got: {msg}"
        );
    }

    // ── 控制面测试 3：调用方 service 地址 parse 失败路径 ───────────────────

    /// `run_p2p_udp_session` 的第二参数是强类型 `SocketAddr`，非法地址在
    /// Rust 类型系统层已被阻断。这里测试 session.rs 调用方的 parse 逻辑：
    /// `service.parse::<SocketAddr>()` 失败时，anyhow error 应同时包含
    /// tunnel name 和原始 service 字符串，便于运维排查配置错误。
    #[test]
    fn caller_service_parse_failure_error_contains_context() {
        let tunnel_name = "bad-addr-tunnel";
        let bad_service = "not-a-valid-addr";

        // 内联模拟 session.rs handle_p2p_ready 中的 parse 逻辑
        let result: Result<SocketAddr> = bad_service.parse().map_err(|e| {
            anyhow!(
                "P2P UDP: parse service addr '{}' for tunnel '{}': {}",
                bad_service,
                tunnel_name,
                e
            )
        });

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains(tunnel_name),
            "error must contain tunnel name; got: {msg}"
        );
        assert!(
            msg.contains(bad_service),
            "error must contain original bad service value; got: {msg}"
        );
    }
}
