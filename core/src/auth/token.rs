use md5::{Digest, Md5};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_MAX_SKEW_SECS: i64 = 15 * 60;

pub fn get_auth_key(token: &str, timestamp: i64) -> String {
    let mut hasher = Md5::new();
    hasher.update(token.as_bytes());
    hasher.update(timestamp.to_string().as_bytes());
    hex::encode(hasher.finalize())
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
    get_auth_key(token, timestamp) == privilege_key
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
        assert!(verify_login_at("", "whatever", 0, 0, 900));
    }
}
