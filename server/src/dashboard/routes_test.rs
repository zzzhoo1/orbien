//! In-process handler tests for the dashboard API.
//!
//! Strategy
//! --------
//! * Build the axum `Router` directly via `routes::router()` — no TCP socket.
//! * Set `DashboardConfig { user: "", password: "" }` so `needs_basic_auth`
//!   returns false and the auth middleware passes every request through.
//! * Use `tower::ServiceExt::oneshot` to drive individual requests.
//! * All assertions are JSON-level (serde_json), not string-matching.
//! * `Service::new(ServerConfig::default())` is safe in CI: when cert/key
//!   paths are empty the transport layer auto-generates an ephemeral
//!   self-signed certificate via rcgen — no filesystem dependency.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::{
        dashboard::{auth::AuthState, routes, DashState},
        service::Service,
    };
    use orbien_core::config::{ServerConfig, DashboardConfig};

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Build a minimal `DashState` backed by an in-memory `Service`.
    /// `user` and `password` are intentionally empty so the auth middleware
    /// treats every request as authenticated (no Basic-Auth challenge).
    fn make_state() -> Arc<DashState> {
        let cfg = ServerConfig::default();
        let svc = Arc::new(Service::new(cfg).expect("Service::new"));
        let web_cfg = DashboardConfig {
            addr: "127.0.0.1".into(),
            port: 0,
            user: String::new(),
            password: String::new(),
            ..Default::default()
        };
        let auth = Some(Arc::new(AuthState::session_only()));
        Arc::new(DashState {
            svc,
            cfg: web_cfg,
            auth,
        })
    }

    /// Send a request through the full middleware stack and collect the body.
    async fn call(state: Arc<DashState>, req: Request<Body>) -> (StatusCode, Value) {
        let app = routes::router(Arc::clone(&state)).layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            crate::dashboard::auth::auth_middleware,
        ));
        let resp = app.oneshot(req).await.expect("oneshot");
        let status = resp.status();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("body collect")
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

    // ── GET /api/v1/clients ───────────────────────────────────────────────────

    #[tokio::test]
    async fn list_clients_empty() {
        let state = make_state();
        let req = Request::get("/api/v1/clients").body(Body::empty()).unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert_eq!(json["data"]["total"], 0);
        assert!(json["data"]["items"].as_array().unwrap().is_empty());
    }

    // ── GET /api/v1/proxies ───────────────────────────────────────────────────

    #[tokio::test]
    async fn list_proxies_empty() {
        let state = make_state();
        let req = Request::get("/api/v1/proxies").body(Body::empty()).unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert_eq!(json["data"]["total"], 0);
        assert!(json["data"]["items"].as_array().unwrap().is_empty());
    }

    // ── GET /api/v1/system/info ───────────────────────────────────────────────

    #[tokio::test]
    async fn system_info_returns_version() {
        let state = make_state();
        let req = Request::get("/api/v1/system/info")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        let ver = json["data"]["version"].as_str().unwrap_or("");
        assert!(!ver.is_empty(), "version should not be empty");
    }

    // ── GET /api/v1/clients/{run_id} — not found ──────────────────────────────

    #[tokio::test]
    async fn get_client_not_found() {
        let state = make_state();
        let req = Request::get("/api/v1/clients/nonexistent-run-id")
            .body(Body::empty())
            .unwrap();
        let (status, _) = call(state, req).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ── POST /api/v1/clients/{run_id}/kick — not found ────────────────────────

    #[tokio::test]
    async fn kick_client_not_found_returns_404_code() {
        let state = make_state();
        let req = Request::post("/api/v1/clients/ghost-client/kick")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_ne!(
            json["code"].as_i64().unwrap_or(200),
            200,
            "expected error code for missing client, got: {json}"
        );
    }

    // ── DELETE /api/v1/proxies/{name} — not found ─────────────────────────────

    #[tokio::test]
    async fn kick_proxy_not_found_returns_error_code() {
        let state = make_state();
        let req = Request::delete("/api/v1/proxies/ghost-proxy")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_ne!(
            json["code"].as_i64().unwrap_or(200),
            200,
            "expected error code for missing proxy, got: {json}"
        );
    }

    // ── GET /api/v1/proxies/{name}/traffic — not found ────────────────────────

    #[tokio::test]
    async fn proxy_traffic_not_found() {
        let state = make_state();
        let req = Request::get("/api/v1/proxies/ghost-proxy/traffic")
            .body(Body::empty())
            .unwrap();
        let (status, _) = call(state, req).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ── GET /api/v1/system/traffic ────────────────────────────────────────────

    #[tokio::test]
    async fn system_traffic_returns_ok() {
        let state = make_state();
        let req = Request::get("/api/v1/system/traffic")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert!(
            json["data"]["history"].is_array(),
            "expected history array, got: {json}"
        );
    }

    // ── GET /api/v1/system/tokens ─────────────────────────────────────────────

    #[tokio::test]
    async fn system_tokens_returns_ok() {
        let state = make_state();
        let req = Request::get("/api/v1/system/tokens")
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert!(
            json["data"]["tokens"].is_array(),
            "expected tokens array, got: {json}"
        );
    }

    // ── auth middleware: unauthenticated request is rejected ──────────────────

    #[tokio::test]
    async fn auth_middleware_rejects_when_password_set() {
        let cfg = ServerConfig::default();
        let svc = Arc::new(Service::new(cfg).expect("Service::new"));
        let web_cfg = DashboardConfig {
            addr: "127.0.0.1".into(),
            port: 0,
            user: "admin".into(),
            password: "secret".into(),
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

    // ── auth middleware: correct Basic-Auth credentials pass ──────────────────

    #[tokio::test]
    async fn auth_middleware_passes_with_correct_basic_auth() {
        use base64::Engine;
        let cfg = ServerConfig::default();
        let svc = Arc::new(Service::new(cfg).expect("Service::new"));
        let web_cfg = DashboardConfig {
            addr: "127.0.0.1".into(),
            port: 0,
            user: "admin".into(),
            password: "secret".into(),
            ..Default::default()
        };
        let state = Arc::new(DashState {
            svc,
            cfg: web_cfg,
            auth: Some(Arc::new(AuthState::session_only())),
        });
        let credentials = base64::engine::general_purpose::STANDARD.encode(b"admin:secret");
        let req = Request::get("/api/v1/clients")
            .header("Authorization", format!("Basic {credentials}"))
            .body(Body::empty())
            .unwrap();
        let (status, json) = call(state, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
    }

    // ── GET /metrics (Prometheus) ─────────────────────────────────────────────

    #[tokio::test]
    async fn prometheus_metrics_exposes_expected_series() {
        let state = make_state();
        let app = routes::router(Arc::clone(&state)).layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            crate::dashboard::auth::auth_middleware,
        ));
        let req = Request::get("/metrics").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.expect("oneshot");
        assert_eq!(resp.status(), StatusCode::OK);
        let ctype = resp
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ctype.contains("text/plain"), "expected text/plain, got {ctype}");
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        // Core aggregate series must be present.
        for series in [
            "orbien_clients_online",
            "orbien_clients_total",
            "orbien_connections_current",
            "orbien_traffic_in_bytes_total",
            "orbien_traffic_out_bytes_total",
        ] {
            assert!(
                text.contains(series),
                "missing series {series} in /metrics output"
            );
        }
    }
}
