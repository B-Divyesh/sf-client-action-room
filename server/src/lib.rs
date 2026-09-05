pub mod auth;
pub mod demo;
pub mod scanner;
pub mod state;
pub mod workspace;

use std::time::Duration;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, MatchedPath, Request, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};
use tracing::info;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: String,
}

#[derive(Serialize)]
struct ReadyResponse {
    status: &'static str,
    database: &'static str,
    malware_scanner: &'static str,
}

pub fn app(state: AppState) -> Router {
    let index = state.dist_dir.join("index.html");
    let static_files = ServeDir::new(state.dist_dir.clone()).fallback(ServeFile::new(index));

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/me", get(workspace::me))
        .route(
            "/api/v1/staff/workspace",
            get(workspace::get_workspace).post(workspace::create_workspace),
        )
        .route("/api/v1/staff/actions", post(workspace::create_action))
        .route(
            "/api/v1/staff/actions/{id}/publish",
            post(workspace::publish_link),
        )
        .route(
            "/api/v1/staff/actions/{id}/reminder",
            post(workspace::schedule_reminder),
        )
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
        .layer(CompressionLayer::new())
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

async fn ready(State(state): State<AppState>) -> Response {
    let database_ready = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();
    let scanner_ready = state.scanner.available().await;
    let status = if database_ready && scanner_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(ReadyResponse {
            status: if status.is_success() {
                "ready"
            } else {
                "degraded"
            },
            database: if database_ready {
                "ready"
            } else {
                "unavailable"
            },
            malware_scanner: if scanner_ready {
                "ready"
            } else {
                "unavailable"
            },
        }),
    )
        .into_response()
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
    let has_visitor_cookie = cookie_value(request.headers(), "car_visitor").is_some();
    let local_host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host.starts_with("localhost") || host.starts_with("127.0.0.1"));
    if !matches!(path.as_str(), "/health" | "/ready") {
        let (bucket, allowance, window) = rate_policy(request.method().as_str(), &path);
        let retry_after = rate_identities(request.headers(), &path)
            .into_iter()
            .find_map(|identity| {
                state
                    .limiter
                    .check(format!("{identity}:{bucket}"), allowance, window)
                    .err()
            });
        if let Some(retry_after) = retry_after {
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
    let should_persist = method != axum::http::Method::GET && path.starts_with("/api/");
    let started = std::time::Instant::now();
    let mut response = next.run(request).await;
    if should_persist && response.status().is_success() {
        if let Err(error) = state.persist_snapshot().await {
            tracing::error!(error = %error, "durable database snapshot failed");
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
        }
    }
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
    if is_html && !has_visitor_cookie {
        let visitor = uuid::Uuid::now_v7();
        if let Ok(value) = HeaderValue::from_str(&format!(
            "car_visitor={visitor}; Path=/; HttpOnly; SameSite=Strict; Max-Age=2592000{}",
            if local_host { "" } else { "; Secure" }
        )) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
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

fn rate_identities(headers: &axum::http::HeaderMap, path: &str) -> Vec<String> {
    let mut identities = Vec::new();
    let mut has_product_cookie = false;
    for name in ["car_visitor", "car_demo", "car_client"] {
        if let Some(value) = cookie_value(headers, name) {
            identities.push(format!("cookie:{value}"));
            has_product_cookie = true;
            break;
        }
    }
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty());
    let has_bearer = bearer.is_some();
    if let Some(value) = bearer {
        identities.push(format!("bearer:{}", hex::encode(Sha256::digest(value))));
    }
    for name in ["x-azure-clientip", "x-real-ip", "x-forwarded-for"] {
        let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) else {
            continue;
        };
        let hops: Vec<_> = value
            .split(',')
            .map(str::trim)
            .filter(|hop| !hop.is_empty())
            .collect();
        if let Some(first) = hops.first() {
            identities.push(format!(
                "{name}:{}",
                first.chars().take(64).collect::<String>()
            ));
        }
        if let Some(last) = (hops.len() > 1).then(|| hops[hops.len() - 1]) {
            identities.push(format!(
                "{name}:{}",
                last.chars().take(64).collect::<String>()
            ));
        }
    }
    if identities.is_empty() || (path.starts_with("/api/") && !has_product_cookie && !has_bearer) {
        identities.push("anonymous-no-cookie".into());
    }
    identities
}

fn cookie_value(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (cookie_name, value) = cookie.trim().split_once('=')?;
                (cookie_name == name).then(|| value.chars().take(80).collect())
            })
        })
}

fn rate_policy(method: &str, path: &str) -> (&'static str, usize, Duration) {
    if method == "POST"
        && matches!(
            path,
            "/api/v1/demo/sessions" | "/api/v1/demo/session/ensure" | "/api/v1/demo/session/reset"
        )
    {
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
        } else if path.starts_with("/assets/") || path.starts_with("/fonts/") {
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

    #[tokio::test]
    async fn anonymous_session_creation_cannot_bypass_limits_by_changing_forwarded_ip() {
        let router = app(test_state().await);
        for attempt in 0..4 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/demo/sessions")
                        .header("host", "localhost:4173")
                        .header("x-forwarded-for", format!("203.0.113.{attempt}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap();
            if attempt < 3 {
                assert_eq!(response.status(), StatusCode::CREATED);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
                assert!(response.headers().contains_key(header::RETRY_AFTER));
            }
        }
    }
}
