use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

#[derive(Clone)]
pub struct AppState {
    build_sha: String,
}

impl AppState {
    pub fn new(build_sha: impl Into<String>) -> Self {
        Self {
            build_sha: build_sha.into(),
        }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: String,
}

pub fn app(build_sha: impl Into<String>) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(AppState::new(build_sha))
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        build_sha: state.build_sha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_reports_the_build_sha() {
        let response = app("test-sha")
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{"status":"ok","build_sha":"test-sha"}"#
        );
    }
}
