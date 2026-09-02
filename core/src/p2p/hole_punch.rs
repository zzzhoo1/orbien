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
//! The token is sent as the first 36 bytes of every probe so each side can
//! verify it is talking to the right peer and discard stray packets.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

/// Configuration for a single hole-punch attempt.
#[derive(Debug, Clone)]
pub struct HolePunchConfig {
    /// Shared token (from `P2pReady.token`) used to verify peer identity.
    pub token: String,
    /// Our own candidate addresses to bind/listen on.
    /// Typically includes LAN address + the WAN address observed by the server.
    pub local_candidates: Vec<SocketAddr>,
    /// Peer's candidate addresses to punch toward.
    pub remote_candidates: Vec<SocketAddr>,
    /// How long to attempt before giving up.
    pub timeout: Duration,
    /// Number of UDP probe packets to send per candidate pair before pausing.
    pub probe_count: u32,
    /// Delay between probe bursts.
    pub probe_interval: Duration,
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
    if let Some(sock) = try_udp_punch(&cfg).await {
        return HolePunchResult::Udp(sock);
    }
    if let Some(stream) = try_tcp_connect(&cfg).await {
        return HolePunchResult::Tcp(stream);
    }
    HolePunchResult::Failed
}

// ── UDP hole-punch ────────────────────────────────────────────────────────────

async fn try_udp_punch(cfg: &HolePunchConfig) -> Option<UdpSocket> {
    if cfg.remote_candidates.is_empty() {
        return None;
    }

    let token_bytes = cfg.token.as_bytes();
    let mut probe = [0u8; 36];
    let copy_len = token_bytes.len().min(36);
    probe[..copy_len].copy_from_slice(&token_bytes[..copy_len]);

    let mut tasks = tokio::task::JoinSet::new();

    for remote in cfg.remote_candidates.clone() {
        let probe = probe;
        let probe_count = cfg.probe_count;
        let probe_interval = cfg.probe_interval;
        let token_bytes_owned = cfg.token.clone();

        tasks.spawn(async move {
            let sock = UdpSocket::bind("0.0.0.0:0").await.ok()?;
            sock.connect(remote).await.ok()?;

            for _ in 0..probe_count {
                let _ = sock.send(&probe).await;
                tokio::time::sleep(probe_interval).await;
            }

            // `timeout(…).await` → Result<Result<usize, io::Error>, Elapsed>
            // First `.ok()?`  discards Elapsed, yielding Option<Result<usize, io::Error>>
            // Second `.ok()?` discards io::Error, yielding Option<usize> and then usize via `?`
            let mut buf = [0u8; 64];
            let n = timeout(Duration::from_secs(5), sock.recv(&mut buf))
                .await
                .ok()?  // discard Elapsed error → Option<Result<usize, io::Error>>
                .ok()?; // discard io::Error     → usize (propagates None on error)
            let received_token = &buf[..n.min(token_bytes_owned.len())];
            if received_token == token_bytes_owned.as_bytes() {
                Some(sock)
            } else {
                None
            }
        });
    }

    let overall = timeout(cfg.timeout, async move {
        while let Some(res) = tasks.join_next().await {
            if let Ok(Some(sock)) = res {
                tasks.abort_all();
                return Some(sock);
            }
        }
        None
    });

    overall.await.ok().flatten()
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
        let cfg = HolePunchConfig {
            token: "tok".into(),
            local_candidates: vec![],
            remote_candidates: parse_candidates("192.0.2.1:9999,192.0.2.2:9999"),
            timeout: Duration::from_millis(200),
            probe_count: 1,
            probe_interval: Duration::from_millis(10),
        };
        assert!(matches!(punch(cfg).await, HolePunchResult::Failed));
    }

    #[tokio::test]
    async fn udp_self_punch_loopback() {
        let token = "loopback-test-token-123456789012".to_string();

        let sock_a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sock_b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr_a = sock_a.local_addr().unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        drop(sock_a);
        drop(sock_b);

        let cfg_a = HolePunchConfig {
            token: token.clone(),
            local_candidates: vec![addr_a],
            remote_candidates: vec![addr_b],
            timeout: Duration::from_secs(5),
            probe_count: 10,
            probe_interval: Duration::from_millis(50),
        };
        let cfg_b = HolePunchConfig {
            token: token.clone(),
            local_candidates: vec![addr_b],
            remote_candidates: vec![addr_a],
            timeout: Duration::from_secs(5),
            probe_count: 10,
            probe_interval: Duration::from_millis(50),
        };

        let (res_a, res_b) = tokio::join!(punch(cfg_a), punch(cfg_b));

        let a_ok = matches!(res_a, HolePunchResult::Udp(_));
        let b_ok = matches!(res_b, HolePunchResult::Udp(_));
        assert!(a_ok || b_ok, "expected at least one UDP punch to succeed on loopback");
    }
}
