//! Session-cookie + WebAuthn + password-login authentication layer.
//!
//! ## Session flow
//! 1. Client POSTs `/api/v1/auth/login` (password) **or** completes a
//!    WebAuthn ceremony via `/api/v1/auth/webauthn/login/finish`.
//! 2. Server mints a random 32-byte session token, stores it in `AuthState`,
//!    and sets `Set-Cookie: orbien_session=<token>; HttpOnly; Path=/; SameSite=Strict`.
//! 3. Every subsequent API request carries that cookie.
//!
//! ## Backward compatibility
//! If no session cookie is present the middleware falls back to HTTP Basic Auth
//! so existing integrations keep working without changes.

use crate::dashboard::DashState;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use rand::RngExt;
use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use webauthn_rs::{
    prelude::{PasskeyAuthentication, PasskeyRegistration, Url},
    Webauthn, WebauthnBuilder,
};

// ── session record ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Session {
    username: String,
    created: Instant,
}

const SESSION_TTL: Duration = Duration::from_secs(8 * 3600);
const COOKIE_NAME: &str = "orbien_session";
const LOGIN_WINDOW: Duration = Duration::from_secs(60);
const LOGIN_MAX_ATTEMPTS: u32 = 8;

use webauthn_rs::prelude::Passkey;

// ── public AuthState shared via DashState ────────────────────────────────────

pub struct AuthState {
    sessions: DashMap<String, Session>,
    passkeys: DashMap<String, Vec<Passkey>>,
    reg_states: DashMap<String, PasskeyRegistration>,
    auth_states: DashMap<String, PasskeyAuthentication>,
    login_attempts: DashMap<String, (u32, Instant)>,
    pub webauthn: Option<Webauthn>,
    /// M3: when set, passkeys are persisted to this JSON file (0600) on every
    /// mutation and reloaded on startup, so registrations survive restarts.
    passkey_store: Option<PathBuf>,
}

impl AuthState {
    pub fn session_only() -> Self {
        Self {
            sessions: DashMap::new(),
            passkeys: DashMap::new(),
            reg_states: DashMap::new(),
            auth_states: DashMap::new(),
            login_attempts: DashMap::new(),
            webauthn: None,
            passkey_store: None,
        }
    }

    pub fn new(rp_id: &str, rp_origin: &str) -> anyhow::Result<Self> {
        let origin = Url::parse(rp_origin)
            .map_err(|e| anyhow::anyhow!("invalid rp_origin {rp_origin}: {e}"))?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)?.build()?;
        let mut this = Self::session_only();
        this.webauthn = Some(webauthn);
        Ok(this)
    }

    /// M3: attach a passkey persistence file and load any existing entries.
    /// Must be called before the state is shared; failures are logged and
    /// non-fatal (dashboard keeps working with in-memory passkeys only).
    pub fn with_passkey_store(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        match std::fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<HashMap<String, Vec<Passkey>>>(&bytes) {
                Ok(map) => {
                    let count: usize = map.values().map(|v| v.len()).sum();
                    for (user, keys) in map {
                        self.passkeys.insert(user, keys);
                    }
                    tracing::info!(path = %path.display(), passkeys = count, "passkey store loaded");
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), "passkey store parse failed, starting empty: {e}")
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(path = %path.display(), "passkey store does not exist yet, starting empty");
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), "passkey store unreadable, starting empty: {e}")
            }
        }
        self.passkey_store = Some(path);
        self
    }

    /// Serialize passkeys to the store file (0600). Called after every
    /// mutation; write failures are logged, never fatal.
    fn persist_passkeys(&self) {
        let Some(path) = &self.passkey_store else {
            return;
        };
        // Clone out of the DashMap so we don't return references into it.
        let map: HashMap<String, Vec<Passkey>> = self
            .passkeys
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();
        match serde_json::to_vec_pretty(&map) {
            Ok(bytes) => {
                // Write to a temp file then rename for atomicity.
                let tmp = path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp, &bytes)
                    .and_then(|_| std::fs::rename(&tmp, path))
                {
                    tracing::warn!(path = %path.display(), "passkey store write failed: {e}");
                    return;
                }
                #[cfg(unix)]
                if let Err(e) = std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600)) {
                    tracing::warn!(path = %path.display(), "passkey store chmod failed: {e}");
                }
            }
            Err(e) => tracing::warn!("passkey store serialize failed: {e}"),
        }
    }

    pub fn webauthn_enabled(&self) -> bool {
        self.webauthn.is_some()
    }

    // ── session helpers ───────────────────────────────────────────────────────

    pub fn create_session(&self, username: &str) -> String {
        let token = random_token();
        self.sessions.insert(
            token.clone(),
            Session {
                username: username.to_string(),
                created: Instant::now(),
            },
        );
        self.evict_expired();
        token
    }

    pub fn validate_session(&self, token: &str) -> Option<String> {
        let entry = self.sessions.get(token)?;
        if entry.created.elapsed() > SESSION_TTL {
            drop(entry);
            self.sessions.remove(token);
            return None;
        }
        Some(entry.username.clone())
    }

    pub fn remove_session(&self, token: &str) {
        self.sessions.remove(token);
    }

    fn evict_expired(&self) {
        self.sessions
            .retain(|_, v| v.created.elapsed() <= SESSION_TTL);
    }

    // ── passkey helpers ───────────────────────────────────────────────────────

    pub fn store_passkey(&self, username: &str, passkey: Passkey) {
        self.passkeys
            .entry(username.to_string())
            .or_default()
            .push(passkey);
        self.persist_passkeys();
    }

    pub fn passkeys_for(&self, username: &str) -> Vec<Passkey> {
        self.passkeys
            .get(username)
            .map(|v| v.value().clone())
            .unwrap_or_default()
    }

    pub fn all_passkeys(&self) -> Vec<Passkey> {
        self.passkeys
            .iter()
            .flat_map(|e| e.value().to_vec())
            .collect()
    }

    #[allow(dead_code)]
    pub fn update_passkey(&self, username: &str, updated: &Passkey) {
        let mut changed = false;
        if let Some(mut entry) = self.passkeys.get_mut(username) {
            for pk in entry.iter_mut() {
                if pk.cred_id() == updated.cred_id() {
                    *pk = updated.clone();
                    changed = true;
                }
            }
        }
        if changed {
            self.persist_passkeys();
        }
    }

    pub fn apply_auth_result(
        &self,
        auth_result: &webauthn_rs::prelude::AuthenticationResult,
    ) -> Option<String> {
        // M3 note: persist AFTER the iter_mut guard is dropped — DashMap shard
        // locks are not reentrant, so calling persist_passkeys() (which takes a
        // read lock) inside the loop would self-deadlock the auth thread.
        let mut persisted_user = None;
        for mut entry in self.passkeys.iter_mut() {
            for pk in entry.value_mut().iter_mut() {
                if auth_result.cred_id() == pk.cred_id() {
                    pk.update_credential(auth_result);
                    persisted_user = Some(entry.key().clone());
                    break;
                }
            }
        }
        if persisted_user.is_some() {
            self.persist_passkeys();
        }
        persisted_user
    }

    // ── pending state helpers ─────────────────────────────────────────────────

    pub fn save_reg_state(&self, username: &str, state: PasskeyRegistration) {
        self.reg_states.insert(username.to_string(), state);
    }

    pub fn take_reg_state(&self, username: &str) -> Option<PasskeyRegistration> {
        self.reg_states.remove(username).map(|(_, v)| v)
    }

    pub fn save_auth_state(&self, key: &str, state: PasskeyAuthentication) {
        self.auth_states.insert(key.to_string(), state);
    }

    pub fn take_auth_state(&self, key: &str) -> Option<PasskeyAuthentication> {
        self.auth_states.remove(key).map(|(_, v)| v)
    }

    pub fn login_allowed(&self, key: &str) -> bool {
        match self.login_attempts.get(key) {
            Some(entry) => {
                let (count, started) = *entry;
                started.elapsed() > LOGIN_WINDOW || count < LOGIN_MAX_ATTEMPTS
            }
            None => true,
        }
    }

    pub fn record_login_failure(&self, key: &str) {
        self.login_attempts
            .entry(key.to_string())
            .and_modify(|(count, started)| {
                if started.elapsed() > LOGIN_WINDOW {
                    *count = 1;
                    *started = Instant::now();
                } else {
                    *count = count.saturating_add(1);
                }
            })
            .or_insert((1, Instant::now()));
        self.login_attempts
            .retain(|_, (_, started)| started.elapsed() <= LOGIN_WINDOW * 2);
    }

    pub fn clear_login_failures(&self, key: &str) {
        self.login_attempts.remove(key);
    }
}

fn random_token() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

// ── Axum middleware ───────────────────────────────────────────────────────────

#[allow(clippy::result_large_err)]
pub async fn auth_middleware(
    State(state): State<Arc<DashState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = req.uri().path();

    if path.starts_with("/api/v1/auth/") || path == "/healthz" {
        return Ok(next.run(req).await);
    }

    if !path.starts_with("/api/") {
        return Ok(next.run(req).await);
    }

    if state.cfg.disable_auth {
        return Ok(next.run(req).await);
    }

    if let Some(auth) = &state.auth {
        if let Some(token) = extract_cookie(req.headers(), COOKIE_NAME) {
            if auth.validate_session(&token).is_some() {
                return Ok(next.run(req).await);
            }
        }
    }

    if needs_basic_auth(&state) && basic_auth_ok(&state, req.headers()) {
        return Ok(next.run(req).await);
    }

    let mut res = (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"Restricted\""),
    );
    Err(res)
}

// ── cookie helpers ────────────────────────────────────────────────────────────

pub fn extract_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let cookie_str = headers.get(header::COOKIE).and_then(|v| v.to_str().ok())?;
    for pair in cookie_str.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix(&format!("{name}=")) {
            return Some(val.to_string());
        }
    }
    None
}

/// 安全开关：直连部署（无可信反代）时必须为 false——X-Forwarded-* 头完全由客户端控制。
/// 仅当 dashboard 部署在可信代理之后（代理会清洗/设置这些头）才置 true。
pub const TRUSTED_PROXY_ENABLED: bool = false;

pub fn cookie_secure(headers: &axum::http::HeaderMap, origin: &str) -> bool {
    // M1 修复：直连模式下 X-Forwarded-Proto 可被客户端伪造以剥离 Cookie Secure 标记，
    // 因此仅显式启用可信代理时才读取转发头。
    if TRUSTED_PROXY_ENABLED {
        if let Some(proto) = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
        {
            return proto
                .split(',')
                .next_back()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("https");
        }
    }
    origin.trim().to_ascii_lowercase().starts_with("https://")
}

pub fn session_cookie(token: &str, clear: bool, secure: bool) -> HeaderValue {
    let secure_flag = if secure { "; Secure" } else { "" };
    if clear {
        HeaderValue::from_str(&format!(
            "{COOKIE_NAME}=; HttpOnly{secure_flag}; Path=/; SameSite=Strict; Max-Age=0"
        ))
        .unwrap()
    } else {
        HeaderValue::from_str(&format!(
            "{COOKIE_NAME}={token}; HttpOnly{secure_flag}; Path=/; SameSite=Strict; Max-Age={}",
            SESSION_TTL.as_secs()
        ))
        .unwrap()
    }
}

pub fn wa_state_cookie(state_key: &str, secure: bool) -> HeaderValue {
    let secure_flag = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "orbien_wa_state={state_key}; HttpOnly{secure_flag}; Path=/api/v1/auth; SameSite=Strict; Max-Age=120"
    ))
    .unwrap()
}

pub fn clear_wa_state_cookie(secure: bool) -> HeaderValue {
    let secure_flag = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "orbien_wa_state=; HttpOnly{secure_flag}; Path=/api/v1/auth; Max-Age=0"
    ))
    .unwrap()
}

// ── basic-auth helpers (kept for backward compat) ─────────────────────────────

fn needs_basic_auth(state: &DashState) -> bool {
    !state.cfg.user.is_empty() || !state.cfg.password.is_empty()
}

pub fn credentials_match(expected_user: &str, expected_pass: &str, user: &str, pass: &str) -> bool {
    constant_time_eq(user.as_bytes(), expected_user.as_bytes())
        && constant_time_eq(pass.as_bytes(), expected_pass.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for i in 0..max {
        let x = *a.get(i).unwrap_or(&0);
        let y = *b.get(i).unwrap_or(&0);
        diff |= (x ^ y) as usize;
    }
    diff == 0
}

fn basic_auth_ok(state: &DashState, headers: &axum::http::HeaderMap) -> bool {
    use base64::Engine;
    let Some(h) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some(b64) = h
        .strip_prefix("Basic ")
        .or_else(|| h.strip_prefix("basic "))
    else {
        return false;
    };
    let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) else {
        return false;
    };
    let Ok(s) = String::from_utf8(raw) else {
        return false;
    };
    let Some((u, p)) = s.split_once(':') else {
        return false;
    };
    credentials_match(&state.cfg.user, &state.cfg.password, u, p)
}

/// M2 修复：直连模式下 X-Forwarded-For 完全可伪造，登录限速键若采信该头即可被
/// 无限速爆破。仅可信代理模式下读取转发头；`peer_ip` 是不可伪造的真实对端地址，
/// 由调用方从 ConnectInfo 传入。
pub fn client_key(headers: &axum::http::HeaderMap, peer_ip: Option<IpAddr>) -> String {
    if TRUSTED_PROXY_ENABLED {
        if let Some(forwarded) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next_back())
            .map(str::trim)
            .filter(|v| v.parse::<IpAddr>().is_ok())
        {
            return forwarded.to_string();
        }
    }
    match peer_ip {
        Some(ip) => format!("direct:{ip}"),
        None => "direct:unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::net::IpAddr;

    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }

    // 直连模式（TRUSTED_PROXY_ENABLED=false）：转发头一律不采信
    #[test]
    fn client_key_no_header() {
        let headers = HeaderMap::new();
        assert_eq!(client_key(&headers, ip("10.0.0.9")), "direct:10.0.0.9");
    }

    #[test]
    fn client_key_direct_mode_ignores_forwarded_for() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "192.168.1.1".parse().unwrap());
        // 伪造的转发头必须被忽略——限速键只由真实对端 IP 决定
        assert_eq!(client_key(&h, ip("203.0.113.7")), "direct:203.0.113.7");
    }

    #[test]
    fn client_key_direct_mode_attacker_cannot_bypass_rate_limit() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "10.0.0.1, 203.0.113.5".parse().unwrap());
        assert_eq!(client_key(&h, ip("198.51.100.9")), "direct:198.51.100.9");
        h.insert("x-forwarded-for", "10.0.0.2, 203.0.113.5".parse().unwrap());
        // 攻击者换任何伪造头，真实对端 IP 不变 → 限速键稳定
        assert_eq!(client_key(&h, ip("198.51.100.9")), "direct:198.51.100.9");
    }

    #[test]
    fn client_key_direct_mode_no_peer_falls_back() {
        let headers = HeaderMap::new();
        assert_eq!(client_key(&headers, None), "direct:unknown");
    }

    #[test]
    fn client_key_direct_mode_invalid_header_still_ignored() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "garbage".parse().unwrap());
        assert_eq!(client_key(&h, ip("192.0.2.1")), "direct:192.0.2.1");
    }

    #[test]
    fn client_key_direct_mode_trims_spaces_in_header_ignored() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "1.2.3.4 ,  203.0.113.5  ".parse().unwrap());
        // 即便格式合法，直连模式也不采信
        assert_eq!(client_key(&h, ip("198.51.100.9")), "direct:198.51.100.9");
    }

    #[test]
    fn cookie_secure_no_header_http() {
        let h = HeaderMap::new();
        assert!(!cookie_secure(&h, "http://example.com"));
    }

    #[test]
    fn cookie_secure_no_header_https() {
        let h = HeaderMap::new();
        assert!(cookie_secure(&h, "https://example.com"));
    }

    // 直连模式：伪造/合法的 x-forwarded-proto 都不影响 Secure 判定
    #[test]
    fn cookie_secure_direct_mode_ignores_forwarded_proto() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-proto", "http, https".parse().unwrap());
        assert!(!cookie_secure(&h, "http://example.com"));
    }

    #[test]
    fn cookie_secure_direct_mode_attacker_cannot_upgrade() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-proto", "https".parse().unwrap());
        assert!(!cookie_secure(&h, "http://example.com"));
    }
}
