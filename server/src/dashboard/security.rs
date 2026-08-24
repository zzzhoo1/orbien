//! Security hardening middleware for the dashboard Web server.
//!
//! Adds a set of conservative security response headers to every response.
//! These are safe to apply globally and do not affect API behavior — they
//! only instruct browsers to enforce stricter security policies.

use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

/// Insert security headers into a response.
fn apply_security_headers(resp: &mut Response) {
    let headers = resp.headers_mut();

    // HSTS — only meaningful over HTTPS; harmless to set otherwise.
    headers
        .entry("strict-transport-security")
        .or_insert(HeaderValue::from_static(
            "max-age=31536000; includeSubDomains",
        ));

    // Prevent clickjacking.
    headers
        .entry("x-frame-options")
        .or_insert(HeaderValue::from_static("SAMEORIGIN"));

    // Prevent MIME-type sniffing.
    headers
        .entry("x-content-type-options")
        .or_insert(HeaderValue::from_static("nosniff"));

    // Limit referrer information leakage.
    headers
        .entry("referrer-policy")
        .or_insert(HeaderValue::from_static("strict-origin-when-cross-origin"));
}

/// Axum middleware that decorates every response with security headers.
pub async fn security_headers(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut resp = next.run(req).await;
    apply_security_headers(&mut resp);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(security_headers))
    }

    #[tokio::test]
    async fn adds_security_headers() {
        let resp = app()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let headers = resp.headers();
        assert_eq!(
            headers.get("x-frame-options").unwrap(),
            "SAMEORIGIN"
        );
        assert_eq!(
            headers.get("x-content-type-options").unwrap(),
            "nosniff"
        );
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            headers.get("strict-transport-security").unwrap(),
            "max-age=31536000; includeSubDomains"
        );
        // Body still intact.
        let bytes = resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        assert_eq!(&bytes[..], b"ok");
    }
}
