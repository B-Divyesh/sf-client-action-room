pub mod auth;
pub mod demo;
pub mod state;

use std::time::Duration;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, MatchedPath, Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: String,
}

pub fn app(state: AppState) -> Router {
    let index = state.dist_dir.join("index.html");
    let static_files = ServeDir::new(state.dist_dir.clone()).fallback(ServeFile::new(index));

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/me", get(me))
        .route("/api/v1/demo/sessions", post(demo::create_session))
        .route("/api/v1/demo/session/ensure", post(demo::ensure_session))
        .route("/api/v1/demo/session", delete(demo::destroy_session))
        .route("/api/v1/demo/session/reset", post(demo::reset_session))
        .route("/api/v1/demo/queue", get(demo::queue))
        .route("/api/v1/demo/actions", post(demo::create_action))
        .route(
            "/api/v1/demo/actions/{id}/publish",
            post(demo::publish_link),
        )
        .route(
            "/api/v1/demo/client-links/expired",
            post(demo::expired_link),
        )
        .route("/api/v1/client-links/exchange", post(demo::exchange_link))
        .route("/api/v1/client/actions", get(demo::client_action))
        .route(
            "/api/v1/client/actions/{id}/submissions",
            post(demo::submit_action),
        )
        .route(
            "/api/v1/client/actions/{id}/choice",
            post(demo::submit_choice),
        )
        .route(
            "/api/v1/client/actions/{id}/upload",
            post(demo::upload_file),
        )
        .route(
            "/api/v1/client/actions/{id}/visit",
            post(demo::record_external_visit),
        )
        .route(
            "/api/v1/demo/actions/{id}/reminder",
            post(demo::schedule_reminder),
        )
        .fallback_service(static_files)
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024 + 16 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_and_rate_limit,
        ))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        build_sha: state.build_sha,
    })
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, demo::ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let (session_id, _) = demo::provision_staff(&state, &claims.oid).await?;
    let cookie = demo::demo_cookie(&session_id, &headers, 2_592_000);
    let mut response = Json(serde_json::json!({
        "id": claims.oid,
        "name": claims.name,
        "email": claims.email
    }))
    .into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| demo::ApiError::internal())?,
    );
    Ok(response)
}

async fn security_and_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();
    let route_template = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_owned())
        .unwrap_or_else(|| path.clone());
    if path != "/health" {
        let ip = client_ip(request.headers());
        let (bucket, allowance, window) = rate_policy(request.method().as_str(), &path);
        let key = format!("{ip}:{bucket}");
        if let Err(retry_after) = state.limiter.check(key, allowance, window) {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "code": "rate_limited",
                    "message": "Too many requests. Wait a moment and try again.",
                    "request_id": uuid::Uuid::now_v7().to_string()
                })),
            )
                .into_response();
            if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
            add_security_headers(response.headers_mut(), &path);
            return response;
        }
    }

    let method = request.method().clone();
    let started = std::time::Instant::now();
    let mut response = next.run(request).await;
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    let known_html_route = matches!(
        path.as_str(),
        "/" | "/demo" | "/client" | "/workspace" | "/auth/callback" | "/privacy" | "/terms"
    );
    if response.status().is_success() && is_html && !known_html_route {
        *response.status_mut() = StatusCode::NOT_FOUND;
    }
    add_security_headers(response.headers_mut(), &path);
    info!(
        method = %method,
        route = %route_template,
        status = response.status().as_u16(),
        duration_ms = started.elapsed().as_millis() as u64,
        "request completed"
    );
    response
}

fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("direct")
        .chars()
        .take(64)
        .collect()
}

fn rate_policy(method: &str, path: &str) -> (&'static str, usize, Duration) {
    if method == "POST" && matches!(path, "/api/v1/demo/sessions" | "/api/v1/demo/session/reset") {
        ("demo-session", 3, Duration::from_secs(60))
    } else if method == "POST" && path == "/api/v1/client-links/exchange" {
        ("link-exchange", 10, Duration::from_secs(60))
    } else if method == "POST" && path.ends_with("/submissions") {
        ("client-submit", 5, Duration::from_secs(60))
    } else if method != "GET" {
        ("demo-write", 30, Duration::from_secs(60))
    } else {
        ("read", 40, Duration::from_secs(1))
    }
}

fn add_security_headers(headers: &mut axum::http::HeaderMap, path: &str) {
    let values = [
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
        (
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=(), payment=()",
        ),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
        (
            HeaderName::from_static("content-security-policy"),
            "default-src 'self'; base-uri 'none'; connect-src 'self' https://sociobotcustomers.ciamlogin.com; font-src 'self'; form-action 'self' https://sociobotcustomers.ciamlogin.com; frame-src https://sociobotcustomers.ciamlogin.com; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'; upgrade-insecure-requests",
        ),
        (
            HeaderName::from_static("strict-transport-security"),
            "max-age=31536000; includeSubDomains",
        ),
    ];
    for (name, value) in values {
        headers.insert(name, HeaderValue::from_static(value));
    }
    headers.insert(
        header::CACHE_CONTROL,
        if path.starts_with("/api/") || path == "/client" {
            HeaderValue::from_static("no-store")
        } else if path.starts_with("/assets/") {
            HeaderValue::from_static("public, max-age=31536000, immutable")
        } else {
            HeaderValue::from_static("public, max-age=300")
        },
    );
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use axum::{body::Body, http::Request};
    use chrono::{TimeZone, Utc};
    use http_body_util::BodyExt;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    use super::*;

    async fn test_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        AppState::new(
            "test-sha",
            pool,
            Some(Utc.with_ymd_and_hms(2026, 8, 28, 14, 0, 0).unwrap()),
            PathBuf::from("../dist"),
        )
    }

    #[tokio::test]
    async fn health_reports_the_build_sha_and_security_headers() {
        let response = app(test_state().await)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert_eq!(
            response.headers()[header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{"status":"ok","build_sha":"test-sha"}"#
        );
    }

    #[tokio::test]
    async fn every_non_health_route_is_rate_limited_with_retry_after() {
        let router = app(test_state().await);
        let mut limited = None;
        for _ in 0..45 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/demo/queue")
                        .header("x-forwarded-for", "203.0.113.4")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited = Some(response);
                break;
            }
        }
        let response = limited.expect("the burst allowance must be enforced");
        assert!(response.headers().contains_key(header::RETRY_AFTER));
    }
}
