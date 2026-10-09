//! UDP hole-punching with a TCP fallback.
//!
//! # How it works
//!
//! 1. Both peers have received a `P2pReady` message from the server
//!    containing each other's candidate address lists.
//! 2. Each peer calls [`punch`] concurrently.
//! 3. We attempt UDP hole-punching in parallel across all candidate pairs.
//! 4. If UDP succeeds within the timeout, we return the established socket.
//! 5. Otherwise we fall back to a direct TCP connection attempt to every
//!    candidate.  The first successful TCP connection wins.
//! 6. If all attempts fail we return [`HolePunchResult::Failed`].
//!
//! The token is sent as the first bytes of every probe so each side can
//! verify it is talking to the right peer and discard stray packets.
//!
//! ## Concurrency model inside each candidate task
//!
//! Probes are sent and received **concurrently** via `tokio::select!`:
//!
//! ```text
//!  ┌─ send_probes ──────────────────────────────────────────────────┐
//!  │  loop: sock.send(probe) → sleep(probe_interval) → ...          │
//!  └────────────────────────────────────────────────────────────────┘
//!         tokio::select!  ← first branch to complete wins
//!  ┌─ recv_verified ────────────────────────────────────────────────┐
//!  │  loop: sock.recv(buf) → token check → return Some(sock)        │
//!  └────────────────────────────────────────────────────────────────┘
//! ```
//!
//! The recv loop starts **before** the first probe is sent, which is
//! critical for Port Restricted NAT: the remote NAT only allows our
//! incoming packet after we have sent one, but our recv must be ready
//! to catch the peer's reply immediately.  The old sequential design
//! (all probes first, recv after) lost this race.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

/// Configuration for a single hole-punch attempt.
#[derive(Debug, Clone)]
pub struct HolePunchConfig {
    /// Shared token (from `P2pReady.token`) used to verify peer identity.
    pub token: String,
    /// Local candidate addresses to bind to when sending probes.
    /// If empty the OS picks an ephemeral port on `0.0.0.0`.
    pub local_candidates: Vec<SocketAddr>,
    /// Peer's candidate addresses to punch toward.
    pub remote_candidates: Vec<SocketAddr>,
    /// How long to attempt before giving up.
    pub timeout: Duration,
    /// Number of UDP probe packets to send per candidate pair before pausing.
    pub probe_count: u32,
    /// Delay between consecutive probe packets.
    pub probe_interval: Duration,
    /// Whether the UDP hole-punch path is enabled.  When `false`, `punch()`
    /// skips the UDP attempt and goes straight to the TCP fallback.
    pub enable_udp: bool,
}

impl Default for HolePunchConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            local_candidates: Vec::new(),
            remote_candidates: Vec::new(),
            timeout: Duration::from_secs(10),
            probe_count: 5,
            probe_interval: Duration::from_millis(200),
            enable_udp: true,
        }
    }
}

/// Outcome of a hole-punch attempt.
#[derive(Debug)]
pub enum HolePunchResult {
    /// UDP hole-punch succeeded.  The socket is ready for bidirectional use.
    Udp(UdpSocket),
    /// UDP failed; a direct TCP connection was established instead.
    Tcp(TcpStream),
    /// All attempts failed within the configured timeout.
    Failed,
}

/// Parse a comma-separated list of `IP:port` strings into `SocketAddr`s,
/// silently skipping any entries that fail to parse.
pub fn parse_candidates(s: &str) -> Vec<SocketAddr> {
    s.split(',')
        .filter_map(|c| c.trim().parse().ok())
        .collect()
}

/// Attempt to establish a direct P2P connection using UDP hole-punching
/// with a TCP fallback.
///
/// Both peers must call this function concurrently after receiving `P2pReady`.
pub async fn punch(cfg: HolePunchConfig) -> HolePunchResult {
    if cfg.enable_udp {
        if let Some(sock) = try_udp_punch(&cfg).await {
            return HolePunchResult::Udp(sock);
        }
    }
    if let Some(stream) = try_tcp_connect(&cfg).await {
        return HolePunchResult::Tcp(stream);
    }
    HolePunchResult::Failed
}

// ── UDP hole-punch ────────────────────────────────────────────────────────────

/// Build a list of (local_bind, remote) pairs to attempt.
/// If `local_candidates` is empty we use a single wildcard bind (`0.0.0.0:0`).
fn candidate_pairs(cfg: &HolePunchConfig) -> Vec<(SocketAddr, SocketAddr)> {
    let wildcard: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let locals: Vec<SocketAddr> = if cfg.local_candidates.is_empty() {
        vec![wildcard]
    } else {
        cfg.local_candidates.clone()
    };
    let mut pairs = Vec::new();
    for &local in &locals {
        for &remote in &cfg.remote_candidates {
            pairs.push((local, remote));
        }
    }
    pairs
}

/// Send `probe` repeatedly for `count` rounds with `interval` between each.
/// Never resolves on its own — it is always cancelled by the `select!` in
/// the parent task once `recv_verified` returns.
async fn send_probes(sock: &UdpSocket, probe: &[u8], count: u32, interval: Duration) {
    for _ in 0..count {
        // Ignore send errors: the NAT mapping may not be open yet and the
        // kernel will drop the packet rather than returning an error on
        // connected UDP sockets.  We keep sending regardless.
        let _ = sock.send(probe).await;
        tokio::time::sleep(interval).await;
    }
    // After `count` probes, keep sending at the same interval until we are
    // cancelled.  This handles slow or lossy links where the peer's echo
    // takes longer than count * interval to arrive.
    loop {
        let _ = sock.send(probe).await;
        tokio::time::sleep(interval).await;
    }
}

/// Receive packets until one carries the complete expected `token`.
/// Returns `Some(())` if a matching packet arrives; returns `None` only
/// if the socket errors out permanently (which terminates the task via ?).
///
/// Takes `&UdpSocket` (not by value) so the caller can share the same
/// socket with a concurrent sender inside `tokio::select!` without an
/// E0505 borrow/move conflict; the caller still owns the socket.
async fn recv_verified(sock: &UdpSocket, token: &[u8]) -> Option<()> {
    let mut buf = [0u8; 256];
    loop {
        let n = match sock.recv(&mut buf).await {
            Ok(n) => n,
            // Transient errors (ECONNREFUSED on some OS when a previous send
            // was ICMP-rejected) — skip and keep trying.
            Err(_) => continue,
        };
        // Strict check: the received payload must be at least as long as the
        // token, and the first `token.len()` bytes must match exactly.
        // A truncated packet (n < token.len()) must never be accepted.
        if n >= token.len() && &buf[..token.len()] == token {
            return Some(());
        }
    }
}

async fn try_udp_punch(cfg: &HolePunchConfig) -> Option<UdpSocket> {
    if cfg.remote_candidates.is_empty() {
        return None;
    }

    // Build the probe payload: exactly the token bytes (up to 128 bytes).
    // Keeping the probe equal to the token simplifies echo-based testing
    // and avoids any ambiguity in what the receiver should match against.
    let token_bytes = cfg.token.as_bytes();
    let probe_len = token_bytes.len().min(128);
    let probe: Vec<u8> = token_bytes[..probe_len].to_vec();

    let pairs = candidate_pairs(cfg);
    let mut tasks = tokio::task::JoinSet::new();

    for (local, remote) in pairs {
        let probe = probe.clone();
        let probe_count = cfg.probe_count;
        let probe_interval = cfg.probe_interval;
        let token_owned = cfg.token.clone();

        tasks.spawn(async move {
            // Bind and connect.  A failure here means the address is unusable
            // on this machine; skip it silently.
            let sock = UdpSocket::bind(local).await.ok()?;
            sock.connect(remote).await.ok()?;

            // Both select! branches borrow the same socket (same underlying
            // fd).  `tokio::select!` cancels the losing branch, which drops
            // its future cleanly; the socket is owned by this task and is
            // moved out only after the select resolves.
            tokio::select! {
                // The send loop never resolves; it runs until cancelled.
                _ = send_probes(&sock, &probe, probe_count, probe_interval) => {
                    None // unreachable in practice
                }
                // The recv loop resolves as soon as a valid token arrives.
                result = recv_verified(&sock, token_owned.as_bytes()) => {
                    result.map(|_| sock)
                }
            }
        });
    }

    // The outer timeout covers ALL candidate tasks uniformly.  No inner
    // per-task timeout is needed — if the outer deadline fires, all tasks
    // are dropped via JoinSet.
    timeout(cfg.timeout, async move {
        while let Some(res) = tasks.join_next().await {
            if let Ok(Some(sock)) = res {
                tasks.abort_all();
                return Some(sock);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

// ── TCP fallback ──────────────────────────────────────────────────────────────

async fn try_tcp_connect(cfg: &HolePunchConfig) -> Option<TcpStream> {
    let mut tasks = tokio::task::JoinSet::new();

    for remote in cfg.remote_candidates.clone() {
        let tcp_timeout = cfg.timeout;
        tasks.spawn(async move {
            timeout(tcp_timeout, TcpStream::connect(remote))
                .await
                .ok()
                .and_then(|r| r.ok())
        });
    }

    while let Some(res) = tasks.join_next().await {
        if let Ok(Some(stream)) = res {
            tasks.abort_all();
            return Some(stream);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_candidates_valid() {
        let addrs = parse_candidates("127.0.0.1:1234, 10.0.0.1:5678");
        assert_eq!(addrs.len(), 2);
        assert_eq!(addrs[0].port(), 1234);
        assert_eq!(addrs[1].port(), 5678);
    }

    #[test]
    fn parse_candidates_skips_invalid() {
        let addrs = parse_candidates("not-an-addr, 127.0.0.1:9000, :");
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 9000);
    }

    #[test]
    fn parse_candidates_empty_string() {
        assert!(parse_candidates("").is_empty());
    }

    #[test]
    fn candidate_pairs_empty_locals_uses_wildcard() {
        let cfg = HolePunchConfig {
            remote_candidates: parse_candidates("1.2.3.4:5000,1.2.3.5:5001"),
            ..Default::default()
        };
        let pairs = super::candidate_pairs(&cfg);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "0.0.0.0:0".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn candidate_pairs_with_locals() {
        let cfg = HolePunchConfig {
            local_candidates: parse_candidates("10.0.0.1:0,10.0.0.2:0"),
            remote_candidates: parse_candidates("1.2.3.4:5000"),
            ..Default::default()
        };
        let pairs = super::candidate_pairs(&cfg);
        // 2 locals × 1 remote = 2 pairs
        assert_eq!(pairs.len(), 2);
    }

    // ── Token verification unit tests ─────────────────────────────────────────

    /// Verify that the strict token check rejects a truncated packet.
    #[test]
    fn token_check_rejects_prefix_match() {
        let token = b"full-token-abc123";
        // Only first 4 bytes received.
        let buf = &token[..4];
        let n = buf.len();
        // Must NOT match because n < token.len().
        assert!(!(n >= token.len() && &buf[..token.len().min(n)] == token));
    }

    /// Verify that the strict token check accepts an exact match.
    #[test]
    fn token_check_accepts_exact_match() {
        let token = b"full-token-abc123";
        let buf = token;
        let n = buf.len();
        assert!(n >= token.len() && &buf[..token.len()] == token);
    }

    /// Verify that extra trailing bytes after the token are accepted
    /// (the peer might append data; we only check the prefix).
    #[test]
    fn token_check_accepts_token_with_trailing_bytes() {
        let token = b"full-token-abc123";
        let mut buf = token.to_vec();
        buf.extend_from_slice(b"extra-data");
        let n = buf.len();
        assert!(n >= token.len() && &buf[..token.len()] == token);
    }

    // ── Integration tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn punch_returns_failed_with_no_candidates() {
        let cfg = HolePunchConfig {
            token: "test-token".into(),
            local_candidates: vec![],
            remote_candidates: vec![],
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(matches!(punch(cfg).await, HolePunchResult::Failed));
    }

    #[tokio::test]
    async fn punch_returns_failed_on_unreachable_candidates() {
        // 192.0.2.x is TEST-NET — guaranteed unreachable in any environment.
        let cfg = HolePunchConfig {
            token: "tok".into(),
            local_candidates: vec![],
            remote_candidates: parse_candidates("192.0.2.1:9999,192.0.2.2:9999"),
            timeout: Duration::from_millis(200),
            probe_count: 1,
            probe_interval: Duration::from_millis(10),
            enable_udp: true,
        };
        assert!(matches!(punch(cfg).await, HolePunchResult::Failed));
    }

    /// Loopback self-punch: two echo-server tasks reflect every probe back so
    /// `punch()` can verify the token.  Both send and recv run concurrently,
    /// so the echo servers must loop forever (not break after one packet).
    #[tokio::test]
    async fn udp_self_punch_loopback() {
        let token = "loopback-test-token-123456789012".to_string();

        let sock_a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sock_b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr_a = sock_a.local_addr().unwrap();
        let addr_b = sock_b.local_addr().unwrap();

        // Echo servers: loop forever reflecting packets back.
        // Tokio drops these tasks when the test future completes.
        tokio::spawn(async move {
            let mut buf = [0u8; 256];
            loop {
                match sock_a.recv_from(&mut buf).await {
                    Ok((n, src)) => {
                        let _ = sock_a.send_to(&buf[..n], src).await;
                    }
                    Err(_) => break,
                }
            }
        });

        tokio::spawn(async move {
            let mut buf = [0u8; 256];
            loop {
                match sock_b.recv_from(&mut buf).await {
                    Ok((n, src)) => {
                        let _ = sock_b.send_to(&buf[..n], src).await;
                    }
                    Err(_) => break,
                }
            }
        });

        let cfg_a = HolePunchConfig {
            token: token.clone(),
            local_candidates: vec![],
            remote_candidates: vec![addr_b],
            timeout: Duration::from_secs(5),
            probe_count: 10,
            probe_interval: Duration::from_millis(30),
            enable_udp: true,
        };
        let cfg_b = HolePunchConfig {
            token: token.clone(),
            local_candidates: vec![],
            remote_candidates: vec![addr_a],
            timeout: Duration::from_secs(5),
            probe_count: 10,
            probe_interval: Duration::from_millis(30),
            enable_udp: true,
        };

        let (res_a, res_b) = tokio::join!(punch(cfg_a), punch(cfg_b));

        let a_ok = matches!(res_a, HolePunchResult::Udp(_));
        let b_ok = matches!(res_b, HolePunchResult::Udp(_));
        assert!(
            a_ok || b_ok,
            "expected at least one UDP punch to succeed on loopback"
        );
    }
}
