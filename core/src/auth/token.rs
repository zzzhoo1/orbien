use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_MAX_SKEW_SECS: i64 = 15 * 60;

/// Compute the privilege_key as HMAC-SHA256(key=token, msg=timestamp).
/// The result is lowercase hex-encoded (64 chars).
pub fn get_auth_key(token: &str, timestamp: i64) -> String {
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).expect("HMAC accepts any key length");
    mac.update(timestamp.to_string().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn verify_login(token: &str, privilege_key: &str, timestamp: i64) -> bool {
    verify_login_at(
        token,
        privilege_key,
        timestamp,
        unix_now(),
        DEFAULT_MAX_SKEW_SECS,
    )
}

pub fn verify_login_at(
    token: &str,
    privilege_key: &str,
    timestamp: i64,
    now: i64,
    max_skew_secs: i64,
) -> bool {
    if token.is_empty() {
        return true;
    }
    if timestamp <= 0 {
        return false;
    }
    if now.abs_diff(timestamp) > max_skew_secs as u64 {
        return false;
    }
    // Constant-time comparison to prevent timing side-channels.
    let expected = get_auth_key(token, timestamp);
    constant_time_eq(expected.as_bytes(), privilege_key.as_bytes())
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
        let key = get_auth_key("secret", ts);
        assert!(verify_login_at("secret", &key, ts, ts, 900));
    }

    #[test]
    fn rejects_stale_timestamp() {
        let ts = 1_700_000_000;
        let key = get_auth_key("secret", ts);
        assert!(!verify_login_at("secret", &key, ts, ts + 901, 900));
    }

    #[test]
    fn empty_token_still_allows() {
        // Empty token: any privilege_key is accepted (no auth configured).
        assert!(verify_login_at("", "whatever", 0, 0, 900));
    }

    #[test]
    fn rejects_wrong_key() {
        let ts = 1_700_000_000;
        let key = get_auth_key("secret", ts);
        // Flip one char — must reject.
        let mut bad = key.clone();
        let idx = bad.len() - 1;
        let last = bad.as_bytes()[idx];
        bad.replace_range(idx.., if last == b'a' { "b" } else { "a" });
        assert!(!verify_login_at("secret", &bad, ts, ts, 900));
    }

    #[test]
    fn rejects_zero_timestamp() {
        let key = get_auth_key("secret", 0);
        assert!(!verify_login_at("secret", &key, 0, 0, 900));
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
