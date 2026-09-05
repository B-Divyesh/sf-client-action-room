use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration as ChronoDuration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    demo::{
        self, ApiError, CreateActionRequest, DemoAction, DemoQueue, LinkResponse, ReminderResponse,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    firm_name: String,
    client_label: String,
    client_actor: String,
}

#[derive(Debug, Serialize)]
pub struct StaffProfile {
    id: String,
    name: String,
    email: String,
    has_workspace: bool,
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StaffProfile>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let has_workspace = owner_workspace_id(&state, &claims.oid).await?.is_some();
    Ok(Json(StaffProfile {
        id: claims.oid,
        name: claims.name,
        email: claims.email,
        has_workspace,
    }))
}

pub async fn get_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoQueue>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    demo::load_queue(&state, &workspace_id).await.map(Json)
}

pub async fn create_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<DemoQueue>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    if let Some(existing) = owner_workspace_id(&state, &claims.oid).await? {
        return Ok((
            StatusCode::OK,
            Json(demo::load_queue(&state, &existing).await?),
        ));
    }
    validate_label(
        &payload.firm_name,
        "Enter a firm name in 80 characters or fewer.",
    )?;
    validate_label(
        &payload.client_label,
        "Enter a client workspace name in 80 characters or fewer.",
    )?;
    validate_label(
        &payload.client_actor,
        "Enter a client name in 80 characters or fewer.",
    )?;

    let now = state.now();
    let organization_id = Uuid::now_v7().to_string();
    let workspace_id = Uuid::now_v7().to_string();
    let staff_label = if claims.name.trim().is_empty() {
        "Workspace owner"
    } else {
        claims.name.trim()
    };
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT INTO organizations (id, owner_oid, name, created_at) VALUES (?, ?, ?, ?)")
        .bind(&organization_id)
        .bind(&claims.oid)
        .bind(payload.firm_name.trim())
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO workspaces
         (id, created_at, expires_at, namespace, organization_id, firm_name, client_label, staff_label, client_actor)
         VALUES (?, ?, ?, 'real', ?, ?, ?, ?, ?)",
    )
    .bind(&workspace_id)
    .bind(now.to_rfc3339())
    .bind((now + ChronoDuration::days(36_500)).to_rfc3339())
    .bind(&organization_id)
    .bind(payload.firm_name.trim())
    .bind(payload.client_label.trim())
    .bind(staff_label)
    .bind(payload.client_actor.trim())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::CREATED,
        Json(demo::load_queue(&state, &workspace_id).await?),
    ))
}

pub async fn create_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateActionRequest>,
) -> Result<(StatusCode, Json<DemoAction>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    let actor = staff_label(&state, &workspace_id).await?;
    let title = payload.title.trim();
    let instructions = payload.instructions.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_title",
            "Name the approval in 120 characters or fewer.",
        ));
    }
    if instructions.is_empty() || instructions.chars().count() > 2_000 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_instructions",
            "Tell the client what to review in 2,000 characters or fewer.",
        ));
    }
    let due_at = chrono::DateTime::parse_from_rfc3339(&payload.due_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_deadline",
                "Choose a valid deadline.",
            )
        })?;
    let now = state.now();
    if due_at <= now || due_at > now + ChronoDuration::days(365) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_deadline",
            "Choose a deadline within the next year.",
        ));
    }
    let action = DemoAction {
        id: Uuid::now_v7().to_string(),
        kind: "approval".into(),
        title: title.into(),
        instructions: instructions.into(),
        due_at: due_at.to_rfc3339(),
        status: "open".into(),
        preview_only: false,
        version: 1,
    };
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO actions
         (id, workspace_id, kind, title, instructions, due_at, status, preview_only, version, created_at)
         VALUES (?, ?, 'approval', ?, ?, ?, 'open', 0, 1, ?)",
    )
    .bind(&action.id)
    .bind(&workspace_id)
    .bind(&action.title)
    .bind(&action.instructions)
    .bind(&action.due_at)
    .bind(now.to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    append_action_audit(&mut tx, &workspace_id, &action.id, &actor, now).await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok((StatusCode::CREATED, Json(action)))
}

pub async fn publish_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
) -> Result<Json<LinkResponse>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    let actor = staff_label(&state, &workspace_id).await?;
    let status: String =
        sqlx::query_scalar("SELECT status FROM actions WHERE id = ? AND workspace_id = ?")
            .bind(&action_id)
            .bind(&workspace_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::NOT_FOUND,
                    "action_not_found",
                    "Choose an action from this workspace.",
                )
            })?;
    if status == "completed" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "already_completed",
            "This action is already complete.",
        ));
    }
    let token = random_token();
    let now = state.now();
    let expires_at = now + ChronoDuration::days(7);
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT INTO client_grants (id, token_digest, workspace_id, action_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(Uuid::now_v7().to_string()).bind(token_digest(&token)).bind(&workspace_id).bind(&action_id)
        .bind(now.to_rfc3339()).bind(expires_at.to_rfc3339()).execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    demo::append_audit(
        &mut tx,
        &workspace_id,
        Some(&action_id),
        "client_link_issued",
        &actor,
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

pub async fn schedule_reminder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
) -> Result<Json<ReminderResponse>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    let actor = staff_label(&state, &workspace_id).await?;
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM actions WHERE id = ? AND workspace_id = ? AND status = 'open'",
    )
    .bind(&action_id)
    .bind(&workspace_id)
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
    sqlx::query("INSERT OR REPLACE INTO reminders (id, workspace_id, action_id, scheduled_for, channel, status, created_at) VALUES (?, ?, ?, ?, 'email', 'scheduled', ?)")
        .bind(Uuid::now_v7().to_string()).bind(&workspace_id).bind(&action_id).bind(scheduled.to_rfc3339()).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|_| ApiError::internal())?;
    demo::append_audit(
        &mut tx,
        &workspace_id,
        Some(&action_id),
        "reminder_scheduled",
        &actor,
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

async fn owner_workspace_id(state: &AppState, oid: &str) -> Result<Option<String>, ApiError> {
    sqlx::query_scalar(
        "SELECT w.id FROM workspaces w
         JOIN organizations o ON o.id = w.organization_id
         WHERE w.namespace = 'real' AND o.owner_oid = ?",
    )
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())
}

async fn require_workspace(state: &AppState, oid: &str) -> Result<String, ApiError> {
    owner_workspace_id(state, oid).await?.ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "workspace_setup_required",
            "Name your firm and first client workspace to begin.",
        )
    })
}

async fn staff_label(state: &AppState, workspace_id: &str) -> Result<String, ApiError> {
    sqlx::query_scalar("SELECT staff_label FROM workspaces WHERE id = ? AND namespace = 'real'")
        .bind(workspace_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::internal())
}

async fn append_action_audit(
    tx: &mut Transaction<'_, Sqlite>,
    workspace_id: &str,
    action_id: &str,
    actor: &str,
    now: chrono::DateTime<Utc>,
) -> Result<(), ApiError> {
    demo::append_audit(
        tx,
        workspace_id,
        Some(action_id),
        "action_created",
        actor,
        None,
        now,
    )
    .await?;
    demo::append_audit(
        tx,
        workspace_id,
        Some(action_id),
        "deadline_set",
        actor,
        None,
        now,
    )
    .await
}

fn validate_label(value: &str, message: &'static str) -> Result<(), ApiError> {
    if !(1..=80).contains(&value.trim().chars().count()) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_workspace",
            message,
        ));
    }
    Ok(())
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn token_digest(token: &str) -> String {
    format!("v1:{}", hex::encode(Sha256::digest(token.as_bytes())))
}
