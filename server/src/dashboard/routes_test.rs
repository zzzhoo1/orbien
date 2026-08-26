#![cfg(test)]
//! Dashboard route integration tests.
//!
//! Strategy
//! --------
//! * Build the axum `Router` directly via `routes::router()` — no TCP socket.
//! * Set `DashboardConfig { disable_auth: true }` so the auth middleware
//!   passes every request through without requiring credentials.
//! * Use `tower::ServiceExt::oneshot` to drive individual requests.
//! * All assertions are JSON-level (serde_json), not string-matching.
//! * `Service::new(ServerConfig::default())` is safe in CI: when cert/key
//!   paths are empty, TLS is not enabled.

use super::auth::AuthState;
use super::model::ApiResponse;
use super::routes;
use super::DashState;
use crate::service::Service;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use orbien_core::config::server::{DashboardConfig, ServerConfig};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Build a minimal `DashState` backed by an in-memory `Service`.
    /// `disable_auth` is set to true so the auth middleware treats every
    /// request as authenticated without requiring credentials.
    fn make_state() -> Arc<DashState> {
        let cfg = ServerConfig::default();
        let svc = Arc::new(Service::new(cfg).expect("Service::new"));
        let web_cfg = DashboardConfig {
            addr: "127.0.0.1".into(),
            port: 0,
            user: String::new(),
            password: String::new(),
            disable_auth: true,
            ..Default::default()
        };
        let auth = Some(Arc::new(AuthState::session_only()));
        Arc::new(DashState {
            svc,
            cfg: web_cfg,
            auth,
        })
    }

    async fn call(state: Arc<DashState>, req: Request<Body>) -> (StatusCode, Value) {
        let router = routes::router(state);
        let resp = router.oneshot(req).await.expect("oneshot");
        let status = resp.status();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    // ── /healthz ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn healthz_returns_ok() {
        let state = make_state();
        let req = Request::get("/healthz").body(Body::empty()).unwrap();
        let (status, _) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
    }

    // ── /api/v1/system/info ───────────────────────────────────────────────────

    #[tokio::test]
    async fn system_info_returns_200() {
        let state = make_state();
        let req = Request::get("/api/v1/system/info")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert!(json["data"]["version"].is_string());
    }

    // ── /api/v1/clients ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_clients_empty() {
        let state = make_state();
        let req = Request::get("/api/v1/clients").body(Body::empty()).unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert_eq!(json["data"]["total"], 0);
    }

    #[tokio::test]
    async fn get_client_not_found() {
        let state = make_state();
        let req = Request::get("/api/v1/clients/no-such-id")
            .body(Body::empty())
            .unwrap();
        let (status, _) = call(state, req).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ── /api/v1/proxies ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_proxies_empty() {
        let state = make_state();
        let req = Request::get("/api/v1/proxies").body(Body::empty()).unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert_eq!(json["data"]["total"], 0);
    }

    #[tokio::test]
    async fn list_proxies_with_page_params() {
        let state = make_state();
        let req = Request::get("/api/v1/proxies?page=1&pageSize=10")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
    }

    #[tokio::test]
    async fn kick_proxy_not_found() {
        let state = make_state();
        let req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/proxies/no-such-proxy")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK); // handler returns 200 with code=404 in body
        assert_eq!(json["code"], 404);
    }

    // ── /api/v1/system/traffic ────────────────────────────────────────────────

    #[tokio::test]
    async fn system_traffic_returns_200() {
        let state = make_state();
        let req = Request::get("/api/v1/system/traffic")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
    }

    // ── /api/v1/system/tokens ─────────────────────────────────────────────────

    #[tokio::test]
    async fn system_tokens_returns_200() {
        let state = make_state();
        let req = Request::get("/api/v1/system/tokens")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
    }

    // ── auth middleware: empty credentials without disable_auth are rejected ──

    #[tokio::test]
    async fn auth_middleware_rejects_when_no_credentials_and_no_disable_auth() {
        let cfg = ServerConfig::default();
        let svc = Arc::new(Service::new(cfg).expect("Service::new"));
        let web_cfg = DashboardConfig {
            addr: "127.0.0.1".into(),
            port: 0,
            user: String::new(),
            password: String::new(),
            disable_auth: false,
            ..Default::default()
        };
        let state = Arc::new(DashState {
            svc,
            cfg: web_cfg,
            auth: Some(Arc::new(AuthState::session_only())),
        });
        let req = Request::get("/api/v1/clients").body(Body::empty()).unwrap();
        let (status, _) = call(state, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
