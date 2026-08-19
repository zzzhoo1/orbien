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
use rand::Rng;
use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use webauthn_rs::{
    prelude::{PasskeyAuthentication, PasskeyRegistration},
    Webauthn, WebauthnBuilder,
};

// ── session record ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Session {
    username: String,
    created: Instant,
}

const SESSION_TTL: Duration = Duration::from_secs(8 * 3600); // 8 h
const COOKIE_NAME: &str = "orbien_session";
const LOGIN_WINDOW: Duration = Duration::from_secs(60);
const LOGIN_MAX_ATTEMPTS: u32 = 8;

use webauthn_rs::prelude::Passkey;

// ── public AuthState shared via DashState ────────────────────────────────────

pub struct AuthState {
    /// token → session
    sessions: DashMap<String, Session>,
    /// username → Vec<Passkey>  (private — access through methods only)
    passkeys: DashMap<String, Vec<Passkey>>,
    /// pending registration states keyed by username
    reg_states: DashMap<String, PasskeyRegistration>,
    /// pending authentication states keyed by a per-request token
    auth_states: DashMap<String, PasskeyAuthentication>,
    /// failed login attempts keyed by client identity
    login_attempts: DashMap<String, (u32, Instant)>,
    pub webauthn: Option<Webauthn>,
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
        }
    }

    pub fn new(rp_id: &str, rp_origin: &str) -> anyhow::Result<Self> {
        let origin = url::Url::parse(rp_origin)
            .map_err(|e| anyhow::anyhow!("invalid rp_origin {rp_origin}: {e}"))?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)?.build()?;
        let mut this = Self::session_only();
        this.webauthn = Some(webauthn);
        Ok(this)
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
    }

    pub fn passkeys_for(&self, username: &str) -> Vec<Passkey> {
        self.passkeys
            .get(username)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn all_passkeys(&self) -> Vec<Passkey> {
        self.passkeys
            .iter()
            .flat_map(|e| e.value().clone())
            .collect()
    }

    pub fn update_passkey(&self, username: &str, updated: &Passkey) {
        if let Some(mut entry) = self.passkeys.get_mut(username) {
            for pk in entry.iter_mut() {
                if pk.cred_id() == updated.cred_id() {
                    *pk = updated.clone();
                }
            }
        }
    }

    /// Update the counter for a just-authenticated credential and return the
    /// username that owns it.  Returns `None` when the credential is not found
    /// (should not happen in normal flow).
    pub fn apply_auth_result(
        &self,
        auth_result: &webauthn_rs::prelude::AuthenticationResult,
    ) -> Option<String> {
        for mut entry in self.passkeys.iter_mut() {
            for pk in entry.value_mut().iter_mut() {
                if auth_result.cred_id() == pk.cred_id() {
                    pk.update_credential(auth_result);
                    return Some(entry.key().clone());
                }
            }
        }
        None
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
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

// ── Axum middleware ───────────────────────────────────────────────────────────

/// Checks for a valid session cookie **or** falls back to HTTP Basic Auth.
/// The `/api/v1/auth/*` routes and `/healthz` are always public.
pub async fn auth_middleware(
    State(state): State<Arc<DashState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = req.uri().path();

    // Always allow auth endpoints and healthz through
    if path.starts_with("/api/v1/auth/") || path == "/healthz" {
        return Ok(next.run(req).await);
    }

    // Also allow static assets through (JS/CSS/fonts for the login page)
    if !path.starts_with("/api/") {
        return Ok(next.run(req).await);
    }

    // 1. Try session cookie
    if let Some(auth) = &state.auth {
        if let Some(token) = extract_cookie(req.headers(), COOKIE_NAME) {
            if auth.validate_session(&token).is_some() {
                return Ok(next.run(req).await);
            }
        }
    }

    // 2. Fall back to Basic Auth (for backward compatibility)
    if !needs_basic_auth(&state) {
        return Ok(next.run(req).await);
    }
    if basic_auth_ok(&state, req.headers()) {
        return Ok(next.run(req).await);
    }

    // 3. Reject
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

pub fn cookie_secure(headers: &axum::http::HeaderMap, origin: &str) -> bool {
    if let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    {
        return proto
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("https");
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

pub fn client_key(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| v.parse::<IpAddr>().is_ok())
        .unwrap_or("direct")
        .to_string()
}
