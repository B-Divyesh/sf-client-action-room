use std::time::Duration;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};
use uuid::Uuid;

use crate::state::AppState;

const DEMO_COOKIE: &str = "car_demo";
const CLIENT_COOKIE: &str = "car_client";

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    code: &'static str,
    message: String,
    request_id: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "not_authorized", message)
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "We could not finish that request. Try again.",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            Json(ApiErrorBody {
                code: self.code,
                message: self.message,
                request_id: Uuid::now_v7().to_string(),
            }),
        )
            .into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DemoAction {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub instructions: String,
    pub due_at: String,
    pub status: String,
    pub preview_only: bool,
    pub version: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuditEvent {
    pub id: String,
    pub action_id: Option<String>,
    pub event_name: String,
    pub actor_label: String,
    pub decision: Option<String>,
    pub occurred_at: String,
}

#[derive(Debug, Serialize)]
pub struct DemoQueue {
    pub firm: &'static str,
    pub workspace: &'static str,
    pub staff_owner: &'static str,
    pub client_actor: &'static str,
    pub time_zone: &'static str,
    pub expires_at: String,
    pub server_now: String,
    pub actions: Vec<DemoAction>,
    pub audit: Vec<AuditEvent>,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub path: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    title: String,
    instructions: String,
    due_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRequest {
    token: String,
}

#[derive(Debug, Serialize)]
pub struct ClientActionResponse {
    pub firm: &'static str,
    pub workspace: &'static str,
    pub client_actor: &'static str,
    pub link_expires_at: String,
    pub action: DemoAction,
    pub submission: Option<SubmissionResponse>,
    pub choices: Vec<ActionChoice>,
    pub external_url: Option<String>,
    pub destination_host: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActionChoice {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceRequest {
    actor_label: String,
    option_key: String,
}

#[derive(Debug, Deserialize)]
pub struct VisitRequest {
    actor_label: String,
}

#[derive(Debug, Serialize)]
pub struct CompletionResponse {
    pub kind: String,
    pub actor_label: String,
    pub detail: String,
    pub occurred_at: String,
    pub destination_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReminderResponse {
    pub scheduled_for: String,
    pub status: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmissionRequest {
    actor_label: String,
    decision: String,
    #[serde(default)]
    comment: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SubmissionResponse {
    pub id: String,
    pub actor_label: String,
    pub decision: String,
    pub comment: String,
    pub occurred_at: String,
    pub replayed: bool,
}

pub async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    state
        .purge_expired()
        .await
        .map_err(|_| ApiError::internal())?;
    let (session_id, queue) = provision(&state).await?;
    Ok(with_cookie(
        StatusCode::CREATED,
        queue,
        demo_cookie(&session_id, &headers, 86_400),
    ))
}

pub async fn ensure_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    state
        .purge_expired()
        .await
        .map_err(|_| ApiError::internal())?;
    if let Some(session_id) = cookie_value(&headers, DEMO_COOKIE) {
        if let Ok(queue) = load_queue(&state, &session_id).await {
            return Ok((StatusCode::OK, Json(queue)).into_response());
        }
    }
    let (session_id, queue) = provision(&state).await?;
    Ok(with_cookie(
        StatusCode::CREATED,
        queue,
        demo_cookie(&session_id, &headers, 86_400),
    ))
}

pub async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoQueue>, ApiError> {
    let session_id = valid_demo_session(&state, &headers).await?;
    load_queue(&state, &session_id).await.map(Json)
}

pub async fn reset_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(old_id) = cookie_value(&headers, DEMO_COOKIE) {
        sqlx::query("DELETE FROM demo_sessions WHERE id = ?")
            .bind(old_id)
            .execute(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    }
    let (session_id, queue) = provision(&state).await?;
    Ok(with_cookie(
        StatusCode::OK,
        queue,
        demo_cookie(&session_id, &headers, 86_400),
    ))
}

pub async fn destroy_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(session_id) = cookie_value(&headers, DEMO_COOKIE) {
        sqlx::query("DELETE FROM demo_sessions WHERE id = ?")
            .bind(session_id)
            .execute(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_cookie(DEMO_COOKIE, &headers))
            .map_err(|_| ApiError::internal())?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_cookie(CLIENT_COOKIE, &headers))
            .map_err(|_| ApiError::internal())?,
    );
    Ok(response)
}

pub async fn publish_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
) -> Result<Json<LinkResponse>, ApiError> {
    let session_id = valid_demo_session(&state, &headers).await?;
    let row = sqlx::query("SELECT kind, status FROM demo_actions WHERE id = ? AND session_id = ?")
        .bind(&action_id)
        .bind(&session_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "action_not_found",
                "That sample action is not available. Reset the demo and try again.",
            )
        })?;
    let kind: String = row.get("kind");
    let status: String = row.get("status");
    if status == "completed" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "already_completed",
            "This action is already complete. Reset the demo to try it again.",
        ));
    }

    let token = random_token();
    let digest = token_digest(&token);
    let now = state.now();
    let expires_at = now + ChronoDuration::days(7);
    let grant_id = Uuid::now_v7().to_string();
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO demo_grants (id, token_digest, session_id, action_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&grant_id)
    .bind(&digest)
    .bind(&session_id)
    .bind(&action_id)
    .bind(now.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&action_id),
        match kind.as_str() {
            "upload" => "upload_link_issued",
            "choice" => "choice_link_issued",
            "external_link" => "external_link_issued",
            _ => "client_link_issued",
        },
        "Theo Grant",
        None,
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;

    Ok(Json(LinkResponse {
        path: format!("/client#access={token}"),
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn create_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateActionRequest>,
) -> Result<(StatusCode, Json<DemoAction>), ApiError> {
    let session_id = valid_demo_session(&state, &headers).await?;
    let title = payload.title.trim();
    let instructions = payload.instructions.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_title",
            "Name the approval in 120 characters or fewer.",
        ));
    }
    if instructions.is_empty() || instructions.chars().count() > 2000 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_instructions",
            "Tell the client what to review in 2,000 characters or fewer.",
        ));
    }
    let due_at = parse_time(&payload.due_at).map_err(|_| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_deadline",
            "Choose a valid deadline.",
        )
    })?;
    let now = state.now();
    if due_at <= now || due_at > now + ChronoDuration::days(14) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_deadline",
            "Choose a deadline within the next 14 days.",
        ));
    }

    let action = DemoAction {
        id: Uuid::now_v7().to_string(),
        kind: "approval".to_owned(),
        title: title.to_owned(),
        instructions: instructions.to_owned(),
        due_at: due_at.to_rfc3339(),
        status: "open".to_owned(),
        preview_only: false,
        version: 1,
    };
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO demo_actions
         (id, session_id, kind, title, instructions, due_at, status, preview_only, version, created_at)
         VALUES (?, ?, 'approval', ?, ?, ?, 'open', 0, 1, ?)",
    )
    .bind(&action.id)
    .bind(&session_id)
    .bind(&action.title)
    .bind(&action.instructions)
    .bind(&action.due_at)
    .bind(now.to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&action.id),
        "action_created",
        "Theo Grant",
        None,
        now,
    )
    .await?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&action.id),
        "deadline_set",
        "Theo Grant",
        None,
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok((StatusCode::CREATED, Json(action)))
}

pub async fn expired_link(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LinkResponse>, ApiError> {
    let session_id = valid_demo_session(&state, &headers).await?;
    let action_id: String = sqlx::query_scalar(
        "SELECT id FROM demo_actions WHERE session_id = ? AND kind = 'approval' LIMIT 1",
    )
    .bind(&session_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let token = random_token();
    let now = state.now();
    let expires_at = now - ChronoDuration::hours(1);
    sqlx::query(
        "INSERT INTO demo_grants (id, token_digest, session_id, action_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(token_digest(&token))
    .bind(&session_id)
    .bind(&action_id)
    .bind((now - ChronoDuration::days(8)).to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    Ok(Json(LinkResponse {
        path: format!("/client#access={token}"),
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn exchange_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Response, ApiError> {
    if payload.token.len() < 32 || payload.token.len() > 128 {
        return Err(expired_error());
    }
    let digest = token_digest(&payload.token);
    let row = sqlx::query(
        "SELECT g.id, g.expires_at, g.revoked_at, s.expires_at AS session_expires_at
         FROM demo_grants g JOIN demo_sessions s ON s.id = g.session_id
         WHERE g.token_digest = ?",
    )
    .bind(digest)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(expired_error)?;
    let now = state.now();
    let expires_at = parse_time(row.get("expires_at"))?;
    let session_expires_at = parse_time(row.get("session_expires_at"))?;
    let revoked_at: Option<String> = row.get("revoked_at");
    if expires_at <= now || session_expires_at <= now || revoked_at.is_some() {
        return Err(expired_error());
    }

    let grant_id: String = row.get("id");
    let client_session_id = Uuid::now_v7().to_string();
    let client_expires_at = [expires_at, now + ChronoDuration::hours(2)]
        .into_iter()
        .min()
        .expect("two dates");
    sqlx::query(
        "INSERT INTO demo_client_sessions (id, grant_id, created_at, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&client_session_id)
    .bind(grant_id)
    .bind(now.to_rfc3339())
    .bind(client_expires_at.to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let response_body = serde_json::json!({ "exchanged": true });
    Ok(with_cookie(
        StatusCode::OK,
        response_body,
        client_cookie(&client_session_id, &headers, 7_200),
    ))
}

pub async fn client_action(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClientActionResponse>, ApiError> {
    let client_id = cookie_value(&headers, CLIENT_COOKIE).ok_or_else(|| {
        ApiError::unauthorized("Open the client link again to view this request.")
    })?;
    let row = client_scope(&state, &client_id).await?;
    let action_id: String = row.get("action_id");
    let action = load_action(&state, &action_id).await?;
    let submission = load_submission(&state, &action_id).await?;
    let choices = sqlx::query(
        "SELECT option_key, label FROM demo_action_options WHERE action_id = ? ORDER BY position",
    )
    .bind(&action_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .into_iter()
    .map(|item| ActionChoice {
        key: item.get("option_key"),
        label: item.get("label"),
    })
    .collect();
    let external =
        sqlx::query("SELECT url, destination_host FROM demo_external_links WHERE action_id = ?")
            .bind(&action_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    Ok(Json(ClientActionResponse {
        firm: "Northline Studio",
        workspace: "Alder Street Bakery launch",
        client_actor: "Maya Chen",
        link_expires_at: row.get("link_expires_at"),
        action,
        submission,
        choices,
        external_url: external.as_ref().map(|item| item.get("url")),
        destination_host: external.map(|item| item.get("destination_host")),
    }))
}

pub async fn submit_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
    Json(payload): Json<SubmissionRequest>,
) -> Result<Json<SubmissionResponse>, ApiError> {
    enforce_same_origin(&headers)?;
    validate_submission(&payload)?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| (8..=128).contains(&value.len()))
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "idempotency_key_required",
                "Refresh the page and send your answer again.",
            )
        })?;
    let client_id = cookie_value(&headers, CLIENT_COOKIE).ok_or_else(|| {
        ApiError::unauthorized("Open the client link again to answer this request.")
    })?;
    let scope = client_scope(&state, &client_id).await?;
    let scoped_action_id: String = scope.get("action_id");
    if scoped_action_id != action_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "outside_link_scope",
            "This client link cannot open that request.",
        ));
    }
    let kind: String = sqlx::query_scalar("SELECT kind FROM demo_actions WHERE id = ?")
        .bind(&action_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    if kind != "approval" {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "wrong_action_type",
            "Use the control shown for this request.",
        ));
    }

    if let Some(mut existing) = load_submission(&state, &action_id).await? {
        existing.replayed = true;
        return Ok(Json(existing));
    }

    let now = state.now();
    let session_id: String = scope.get("session_id");
    let grant_id: String = scope.get("grant_id");
    let request_hash = hex::encode(Sha256::digest(
        serde_json::to_vec(&payload).map_err(|_| ApiError::internal())?,
    ));
    let submission_id = Uuid::now_v7().to_string();
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;

    let insert = sqlx::query(
        "INSERT INTO demo_submissions
         (id, session_id, action_id, grant_id, actor_label, decision, comment, idempotency_key, request_hash, occurred_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&submission_id)
    .bind(&session_id)
    .bind(&action_id)
    .bind(&grant_id)
    .bind(payload.actor_label.trim())
    .bind(&payload.decision)
    .bind(payload.comment.trim())
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(now.to_rfc3339())
    .execute(&mut *tx)
    .await;

    if insert.is_err() {
        tx.rollback().await.ok();
        if let Some(mut existing) = load_submission(&state, &action_id).await? {
            existing.replayed = true;
            return Ok(Json(existing));
        }
        return Err(ApiError::internal());
    }

    sqlx::query("UPDATE demo_actions SET status = 'completed', version = version + 1 WHERE id = ? AND status = 'open'")
        .bind(&action_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal())?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&action_id),
        "client_decision_recorded",
        payload.actor_label.trim(),
        Some(&payload.decision),
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;

    Ok(Json(SubmissionResponse {
        id: submission_id,
        actor_label: payload.actor_label.trim().to_owned(),
        decision: payload.decision,
        comment: payload.comment.trim().to_owned(),
        occurred_at: now.to_rfc3339(),
        replayed: false,
    }))
}

pub async fn submit_choice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
    Json(payload): Json<ChoiceRequest>,
) -> Result<Json<CompletionResponse>, ApiError> {
    enforce_same_origin(&headers)?;
    let (session_id, grant_id) = scoped_client(&state, &headers, &action_id, "choice").await?;
    validate_actor(&payload.actor_label)?;
    let label: String = sqlx::query_scalar(
        "SELECT label FROM demo_action_options WHERE action_id = ? AND option_key = ?",
    )
    .bind(&action_id)
    .bind(&payload.option_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_choice",
            "Choose one of the listed options.",
        )
    })?;
    let now = state.now();
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT OR IGNORE INTO demo_choice_submissions (id, session_id, action_id, grant_id, actor_label, option_key, option_label, occurred_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(Uuid::now_v7().to_string()).bind(&session_id).bind(&action_id).bind(&grant_id)
        .bind(payload.actor_label.trim()).bind(&payload.option_key).bind(&label).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    complete_action(
        &mut tx,
        &session_id,
        &action_id,
        "client_choice_recorded",
        payload.actor_label.trim(),
        Some(&label),
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(CompletionResponse {
        kind: "choice".into(),
        actor_label: payload.actor_label.trim().into(),
        detail: label,
        occurred_at: now.to_rfc3339(),
        destination_url: None,
    }))
}

pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<CompletionResponse>, ApiError> {
    enforce_same_origin(&headers)?;
    let (session_id, grant_id) = scoped_client(&state, &headers, &action_id, "upload").await?;
    let mut actor = String::new();
    let mut filename = String::new();
    let mut bytes = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_upload",
            "Choose a PDF file under 5 MB.",
        )
    })? {
        match field.name().unwrap_or_default() {
            "actor_label" => actor = field.text().await.unwrap_or_default(),
            "file" => {
                filename = field.file_name().unwrap_or("upload.pdf").to_owned();
                bytes = field
                    .bytes()
                    .await
                    .map_err(|_| {
                        ApiError::new(
                            StatusCode::BAD_REQUEST,
                            "invalid_upload",
                            "The file could not be read.",
                        )
                    })?
                    .to_vec();
            }
            _ => {}
        }
    }
    validate_actor(&actor)?;
    if bytes.is_empty() || bytes.len() > 5 * 1024 * 1024 || !bytes.starts_with(b"%PDF-") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsafe_file_type",
            "Upload one PDF file under 5 MB.",
        ));
    }
    if bytes
        .windows(5)
        .any(|part| part.eq_ignore_ascii_case(b"EICAR"))
    {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "malware_detected",
            "The safety scan rejected this file. Choose a different PDF.",
        ));
    }
    let now = state.now();
    let checksum = hex::encode(Sha256::digest(&bytes));
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT OR IGNORE INTO demo_uploads (id, session_id, action_id, grant_id, actor_label, original_filename, detected_mime, byte_size, checksum_sha256, scan_state, content, expires_at, occurred_at) VALUES (?, ?, ?, ?, ?, ?, 'application/pdf', ?, ?, 'clean', ?, ?, ?)")
        .bind(Uuid::now_v7().to_string()).bind(&session_id).bind(&action_id).bind(&grant_id).bind(actor.trim())
        .bind(&filename).bind(bytes.len() as i64).bind(&checksum).bind(bytes).bind((now + ChronoDuration::hours(24)).to_rfc3339()).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    complete_action(
        &mut tx,
        &session_id,
        &action_id,
        "client_file_scanned",
        actor.trim(),
        Some("Clean PDF"),
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(CompletionResponse {
        kind: "upload".into(),
        actor_label: actor.trim().into(),
        detail: format!("{filename} · safety scan passed"),
        occurred_at: now.to_rfc3339(),
        destination_url: None,
    }))
}

pub async fn record_external_visit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
    Json(payload): Json<VisitRequest>,
) -> Result<Json<CompletionResponse>, ApiError> {
    enforce_same_origin(&headers)?;
    let (session_id, grant_id) =
        scoped_client(&state, &headers, &action_id, "external_link").await?;
    validate_actor(&payload.actor_label)?;
    let row =
        sqlx::query("SELECT url, destination_host FROM demo_external_links WHERE action_id = ?")
            .bind(&action_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    let url: String = row.get("url");
    let host: String = row.get("destination_host");
    if !url.starts_with("https://") || host == "localhost" || host.starts_with("127.") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsafe_destination",
            "This destination is not a public HTTPS page.",
        ));
    }
    let now = state.now();
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT OR IGNORE INTO demo_external_visits (id, session_id, action_id, grant_id, actor_label, destination_host, occurred_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(Uuid::now_v7().to_string()).bind(&session_id).bind(&action_id).bind(&grant_id).bind(payload.actor_label.trim()).bind(&host).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    complete_action(
        &mut tx,
        &session_id,
        &action_id,
        "external_link_opened",
        payload.actor_label.trim(),
        Some(&host),
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(CompletionResponse {
        kind: "external_link".into(),
        actor_label: payload.actor_label.trim().into(),
        detail: format!("Opened {host}"),
        occurred_at: now.to_rfc3339(),
        destination_url: Some(url),
    }))
}

pub async fn schedule_reminder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
) -> Result<Json<ReminderResponse>, ApiError> {
    let session_id = valid_demo_session(&state, &headers).await?;
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM demo_actions WHERE id = ? AND session_id = ? AND status = 'open'",
    )
    .bind(&action_id)
    .bind(&session_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    if exists == 0 {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "action_not_found",
            "Choose an open action to remind.",
        ));
    }
    let now = state.now();
    let scheduled = now + ChronoDuration::hours(1);
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT OR REPLACE INTO demo_reminders (id, session_id, action_id, scheduled_for, channel, status, created_at) VALUES (?, ?, ?, ?, 'email', 'scheduled', ?)")
        .bind(Uuid::now_v7().to_string()).bind(&session_id).bind(&action_id).bind(scheduled.to_rfc3339()).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&action_id),
        "reminder_scheduled",
        "Theo Grant",
        None,
        now,
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(ReminderResponse {
        scheduled_for: scheduled.to_rfc3339(),
        status: "scheduled",
    }))
}

async fn scoped_client(
    state: &AppState,
    headers: &HeaderMap,
    action_id: &str,
    kind: &str,
) -> Result<(String, String), ApiError> {
    let client_id = cookie_value(headers, CLIENT_COOKIE).ok_or_else(|| {
        ApiError::unauthorized("Open the client link again to answer this request.")
    })?;
    let scope = client_scope(state, &client_id).await?;
    let scoped_action: String = scope.get("action_id");
    if scoped_action != action_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "outside_link_scope",
            "This client link cannot open that request.",
        ));
    }
    let actual_kind: String = sqlx::query_scalar("SELECT kind FROM demo_actions WHERE id = ?")
        .bind(action_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    if actual_kind != kind {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "wrong_action_type",
            "Use the control shown for this request.",
        ));
    }
    Ok((scope.get("session_id"), scope.get("grant_id")))
}

async fn complete_action(
    tx: &mut Transaction<'_, Sqlite>,
    session_id: &str,
    action_id: &str,
    event: &str,
    actor: &str,
    detail: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(), ApiError> {
    sqlx::query("UPDATE demo_actions SET status = 'completed', version = version + 1 WHERE id = ? AND status = 'open'")
        .bind(action_id).execute(&mut **tx).await.map_err(|_| ApiError::internal())?;
    append_audit(tx, session_id, Some(action_id), event, actor, detail, now).await
}

fn validate_actor(actor: &str) -> Result<(), ApiError> {
    if !(1..=80).contains(&actor.trim().chars().count()) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_actor_label",
            "Enter the name that should appear in the record.",
        ));
    }
    Ok(())
}

async fn provision(state: &AppState) -> Result<(String, DemoQueue), ApiError> {
    provision_with_lifetime(state, ChronoDuration::hours(24)).await
}

pub async fn provision_staff(state: &AppState, oid: &str) -> Result<(String, DemoQueue), ApiError> {
    if let Some(session_id) = sqlx::query_scalar::<_, String>(
        "SELECT session_id FROM staff_workspaces WHERE entra_oid = ?",
    )
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    {
        if let Ok(queue) = load_queue(state, &session_id).await {
            return Ok((session_id, queue));
        }
    }
    let (session_id, queue) = provision_with_lifetime(state, ChronoDuration::days(3650)).await?;
    sqlx::query("INSERT OR REPLACE INTO staff_workspaces (entra_oid, session_id, created_at) VALUES (?, ?, ?)")
        .bind(oid).bind(&session_id).bind(state.now().to_rfc3339()).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    Ok((session_id, queue))
}

async fn provision_with_lifetime(
    state: &AppState,
    lifetime: ChronoDuration,
) -> Result<(String, DemoQueue), ApiError> {
    let now = state.now();
    let session_id = Uuid::now_v7().to_string();
    let expires_at = now + lifetime;
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT INTO demo_sessions (id, created_at, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(now.to_rfc3339())
        .bind(expires_at.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal())?;

    let actions = [
        (
            "upload",
            "Upload the signed allergen sheet",
            "Add the signed allergen sheet for the launch file.",
            now - ChronoDuration::days(1),
            false,
        ),
        (
            "approval",
            "Approve the final menu proof",
            "Check the breakfast prices and approve this proof, or ask for a change.",
            now + ChronoDuration::hours(4),
            false,
        ),
        (
            "choice",
            "Choose the launch photo crop",
            "Choose one crop for the launch announcement.",
            now + ChronoDuration::days(1),
            false,
        ),
        (
            "external_link",
            "Open the launch invoice",
            "Open the hosted invoice when it is ready.",
            now + ChronoDuration::days(3),
            false,
        ),
    ];
    let mut approval_id = String::new();
    let mut choice_id = String::new();
    let mut external_id = String::new();
    for (kind, title, instructions, due_at, preview_only) in actions {
        let id = Uuid::now_v7().to_string();
        if kind == "approval" {
            approval_id.clone_from(&id);
        } else if kind == "choice" {
            choice_id.clone_from(&id);
        } else if kind == "external_link" {
            external_id.clone_from(&id);
        }
        sqlx::query(
            "INSERT INTO demo_actions
             (id, session_id, kind, title, instructions, due_at, status, preview_only, version, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 'open', ?, 1, ?)",
        )
        .bind(&id)
        .bind(&session_id)
        .bind(kind)
        .bind(title)
        .bind(instructions)
        .bind(due_at.to_rfc3339())
        .bind(i64::from(preview_only))
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal())?;
    }
    for (key, label, position) in [
        ("wide", "Wide counter crop", 1_i64),
        ("square", "Square pastry crop", 2_i64),
        ("portrait", "Portrait storefront crop", 3_i64),
    ] {
        sqlx::query("INSERT INTO demo_action_options (action_id, option_key, label, position) VALUES (?, ?, ?, ?)")
            .bind(&choice_id).bind(key).bind(label).bind(position)
            .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    }
    sqlx::query(
        "INSERT INTO demo_external_links (action_id, url, destination_host) VALUES (?, ?, ?)",
    )
    .bind(&external_id)
    .bind("https://example.com/")
    .bind("example.com")
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&approval_id),
        "action_created",
        "Theo Grant",
        None,
        now - ChronoDuration::minutes(4),
    )
    .await?;
    append_audit(
        &mut tx,
        &session_id,
        Some(&approval_id),
        "deadline_set",
        "Theo Grant",
        None,
        now - ChronoDuration::minutes(3),
    )
    .await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    let queue = load_queue(state, &session_id).await?;
    Ok((session_id, queue))
}

pub(crate) async fn load_queue(state: &AppState, session_id: &str) -> Result<DemoQueue, ApiError> {
    let expires_at: String =
        sqlx::query_scalar("SELECT expires_at FROM demo_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::GONE,
                    "demo_expired",
                    "This sample room has expired. Reset the demo to open a fresh copy.",
                )
            })?;
    if parse_time(&expires_at)? <= state.now() {
        return Err(ApiError::new(
            StatusCode::GONE,
            "demo_expired",
            "This sample room has expired. Reset the demo to open a fresh copy.",
        ));
    }

    let rows = sqlx::query(
        "SELECT id, kind, title, instructions, due_at, status, preview_only, version
         FROM demo_actions WHERE session_id = ? ORDER BY due_at ASC, id ASC",
    )
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let actions = rows
        .into_iter()
        .map(|row| DemoAction {
            id: row.get("id"),
            kind: row.get("kind"),
            title: row.get("title"),
            instructions: row.get("instructions"),
            due_at: row.get("due_at"),
            status: row.get("status"),
            preview_only: row.get::<i64, _>("preview_only") != 0,
            version: row.get("version"),
        })
        .collect();
    let audit_rows = sqlx::query(
        "SELECT id, action_id, event_name, actor_label, decision, occurred_at
         FROM demo_audit_events WHERE session_id = ? ORDER BY occurred_at ASC, id ASC",
    )
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let audit = audit_rows
        .into_iter()
        .map(|row| AuditEvent {
            id: row.get("id"),
            action_id: row.get("action_id"),
            event_name: row.get("event_name"),
            actor_label: row.get("actor_label"),
            decision: row.get("decision"),
            occurred_at: row.get("occurred_at"),
        })
        .collect();

    Ok(DemoQueue {
        firm: "Northline Studio",
        workspace: "Alder Street Bakery launch",
        staff_owner: "Theo Grant",
        client_actor: "Maya Chen",
        time_zone: "America/New_York",
        expires_at,
        server_now: state.now().to_rfc3339(),
        actions,
        audit,
    })
}

async fn valid_demo_session(state: &AppState, headers: &HeaderMap) -> Result<String, ApiError> {
    let session_id = cookie_value(headers, DEMO_COOKIE).ok_or_else(|| {
        ApiError::unauthorized("Open the demo again to create a fresh sample room.")
    })?;
    let expires_at: Option<String> =
        sqlx::query_scalar("SELECT expires_at FROM demo_sessions WHERE id = ?")
            .bind(&session_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    match expires_at {
        Some(value) if parse_time(&value)? > state.now() => Ok(session_id),
        _ => Err(ApiError::new(
            StatusCode::GONE,
            "demo_expired",
            "This sample room has expired. Reset the demo to open a fresh copy.",
        )),
    }
}

async fn client_scope(
    state: &AppState,
    client_id: &str,
) -> Result<sqlx::sqlite::SqliteRow, ApiError> {
    let row = sqlx::query(
        "SELECT cs.expires_at AS client_expires_at, g.id AS grant_id, g.action_id,
                g.session_id, g.expires_at AS link_expires_at, g.revoked_at,
                ds.expires_at AS demo_expires_at
         FROM demo_client_sessions cs
         JOIN demo_grants g ON g.id = cs.grant_id
         JOIN demo_sessions ds ON ds.id = g.session_id
         WHERE cs.id = ?",
    )
    .bind(client_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::unauthorized("Open the client link again to view this request."))?;
    let now = state.now();
    let revoked_at: Option<String> = row.get("revoked_at");
    let active = parse_time(row.get("client_expires_at"))? > now
        && parse_time(row.get("link_expires_at"))? > now
        && parse_time(row.get("demo_expires_at"))? > now
        && revoked_at.is_none();
    if !active {
        return Err(expired_error());
    }
    Ok(row)
}

async fn load_action(state: &AppState, action_id: &str) -> Result<DemoAction, ApiError> {
    let row = sqlx::query(
        "SELECT id, kind, title, instructions, due_at, status, preview_only, version
         FROM demo_actions WHERE id = ?",
    )
    .bind(action_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(expired_error)?;
    Ok(DemoAction {
        id: row.get("id"),
        kind: row.get("kind"),
        title: row.get("title"),
        instructions: row.get("instructions"),
        due_at: row.get("due_at"),
        status: row.get("status"),
        preview_only: row.get::<i64, _>("preview_only") != 0,
        version: row.get("version"),
    })
}

async fn load_submission(
    state: &AppState,
    action_id: &str,
) -> Result<Option<SubmissionResponse>, ApiError> {
    let row = sqlx::query(
        "SELECT id, actor_label, decision, comment, occurred_at
         FROM demo_submissions WHERE action_id = ? ORDER BY occurred_at ASC LIMIT 1",
    )
    .bind(action_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(row.map(|row| SubmissionResponse {
        id: row.get("id"),
        actor_label: row.get("actor_label"),
        decision: row.get("decision"),
        comment: row.get("comment"),
        occurred_at: row.get("occurred_at"),
        replayed: false,
    }))
}

async fn append_audit(
    tx: &mut Transaction<'_, Sqlite>,
    session_id: &str,
    action_id: Option<&str>,
    event_name: &str,
    actor_label: &str,
    decision: Option<&str>,
    occurred_at: DateTime<Utc>,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO demo_audit_events
         (id, session_id, action_id, event_name, actor_label, decision, occurred_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(session_id)
    .bind(action_id)
    .bind(event_name)
    .bind(actor_label)
    .bind(decision)
    .bind(occurred_at.to_rfc3339())
    .execute(&mut **tx)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(())
}

fn validate_submission(payload: &SubmissionRequest) -> Result<(), ApiError> {
    let actor_len = payload.actor_label.trim().chars().count();
    if !(1..=80).contains(&actor_len) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_actor_label",
            "Enter the name that should appear in the record.",
        ));
    }
    if !matches!(payload.decision.as_str(), "approved" | "changes_requested") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_decision",
            "Choose approve or ask for changes.",
        ));
    }
    if payload.comment.chars().count() > 1000 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "comment_too_long",
            "Keep the note under 1,000 characters.",
        ));
    }
    if payload.decision == "changes_requested" && payload.comment.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "comment_required",
            "Say what needs to change, then send the answer again.",
        ));
    }
    Ok(())
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, ApiError> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| ApiError::internal())
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn token_digest(token: &str) -> String {
    format!("v1:{}", hex::encode(Sha256::digest(token.as_bytes())))
}

fn expired_error() -> ApiError {
    ApiError::new(
        StatusCode::GONE,
        "client_link_expired",
        "This client link has expired. Ask Northline Studio for a new link.",
    )
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (cookie_name, value) = cookie.trim().split_once('=')?;
                (cookie_name == name).then(|| value.to_owned())
            })
        })
}

pub(crate) fn demo_cookie(id: &str, headers: &HeaderMap, max_age: u64) -> String {
    cookie(DEMO_COOKIE, id, headers, max_age)
}

fn client_cookie(id: &str, headers: &HeaderMap, max_age: u64) -> String {
    cookie(CLIENT_COOKIE, id, headers, max_age)
}

fn cookie(name: &str, value: &str, headers: &HeaderMap, max_age: u64) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={max_age}{}",
        secure_attribute(headers)
    )
}

fn clear_cookie(name: &str, headers: &HeaderMap) -> String {
    format!(
        "{name}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
        secure_attribute(headers)
    )
}

fn secure_attribute(headers: &HeaderMap) -> &'static str {
    let local = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host.starts_with("localhost") || host.starts_with("127.0.0.1"));
    if local {
        ""
    } else {
        "; Secure"
    }
}

fn enforce_same_origin(headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(());
    };
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let origin_host = origin
        .split_once("://")
        .map(|(_, value)| value.trim_end_matches('/'))
        .unwrap_or_default();
    if origin_host != host {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "cross_origin_denied",
            "Open the client link again before sending your answer.",
        ));
    }
    Ok(())
}

fn with_cookie<T: Serialize>(status: StatusCode, value: T, cookie: String) -> Response {
    let mut response = (status, Json(value)).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

#[allow(dead_code)]
fn _ttl_contract() -> (Duration, Option<DateTime<Utc>>) {
    (
        Duration::from_secs(86_400),
        Utc::now().checked_add_days(Days::new(1)),
    )
}
