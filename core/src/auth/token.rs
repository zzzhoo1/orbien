use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_MAX_SKEW_SECS: i64 = 15 * 60;

/// Compute the auth_digest as HMAC-SHA256(key=token, msg=timestamp).
/// The result is lowercase hex-encoded (64 chars).
pub fn compute_auth_digest(token: &str, timestamp: i64) -> String {
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).expect("HMAC-SHA256 accepts any key length");
    mac.update(timestamp.to_string().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn verify_login(token: &str, auth_digest: &str, timestamp: i64) -> bool {
    verify_login_at(
        token,
        auth_digest,
        timestamp,
        unix_now(),
        DEFAULT_MAX_SKEW_SECS,
    )
}

/// Alias kept for upstream API compatibility.
pub fn verify_auth_digest(token: &str, auth_digest: &str, timestamp: i64) -> bool {
    verify_login(token, auth_digest, timestamp)
}

/// Verification with injectable clock and skew window (unit-testable).
///
/// Security hardening over plain HMAC comparison:
/// - rejects when token is empty (unconfigured server must deny all);
/// - rejects non-positive timestamps;
/// - rejects timestamps outside `max_skew_secs` of `now` (replay protection);
/// - compares digests in constant time (timing side-channel protection).
pub fn verify_login_at(
    token: &str,
    auth_digest: &str,
    timestamp: i64,
    now: i64,
    max_skew_secs: i64,
) -> bool {
    // Reject when no token is configured — an unconfigured server must not
    // grant access to any client.
    if token.is_empty() {
        return false;
    }
    if auth_digest.is_empty() {
        return false;
    }
    if timestamp <= 0 {
        return false;
    }
    if now.abs_diff(timestamp) > max_skew_secs as u64 {
        return false;
    }
    // Constant-time comparison to prevent timing side-channels.
    let expected = compute_auth_digest(token, timestamp);
    constant_time_eq(expected.as_bytes(), auth_digest.as_bytes())
}

/// Constant-time byte slice comparison (prevents timing oracle).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_current_timestamp() {
        let ts = 1_700_000_000;
        let key = compute_auth_digest("secret", ts);
        assert!(verify_login_at("secret", &key, ts, ts, 900));
    }

    #[test]
    fn rejects_stale_timestamp() {
        let ts = 1_700_000_000;
        let key = compute_auth_digest("secret", ts);
        assert!(!verify_login_at("secret", &key, ts, ts + 901, 900));
    }

    #[test]
    fn empty_token_rejects() {
        // Empty token: authentication must fail (no auth configured = deny all).
        assert!(!verify_login_at(
            "",
            "whatever",
            1_700_000_000,
            1_700_000_000,
            900
        ));
    }

    #[test]
    fn rejects_wrong_key() {
        let ts = 1_700_000_000;
        let key = compute_auth_digest("secret", ts);
        // Flip one char — must reject.
        let mut bad = key.clone();
        let idx = bad.len() - 1;
        let last = bad.as_bytes()[idx];
        bad.replace_range(idx.., if last == b'a' { "b" } else { "a" });
        assert!(!verify_login_at("secret", &bad, ts, ts, 900));
    }

    #[test]
    fn rejects_zero_timestamp() {
        let key = compute_auth_digest("secret", 0);
        assert!(!verify_login_at("secret", &key, 0, 0, 900));
    }

    #[test]
    fn rejects_empty_digest() {
        let ts = 1_700_000_000;
        assert!(!verify_login_at("secret", "", ts, ts, 900));
    }

    #[test]
    fn constant_time_eq_same() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn constant_time_eq_diff_len() {
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
