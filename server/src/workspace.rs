use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration as ChronoDuration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
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
    organization_id: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdditionalWorkspaceRequest {
    client_label: String,
    client_actor: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    name: String,
    time_zone: String,
    retention_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteOrganizationRequest {
    confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    role: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInviteRequest {
    token: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceSummary {
    id: String,
    client_label: String,
    client_actor: String,
    open_actions: i64,
}

#[derive(Debug, Serialize)]
pub struct MemberSummary {
    oid: String,
    display_name: String,
    role: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionSummary {
    tier: String,
    status: String,
    period_end: Option<String>,
    verified_at: String,
}

#[derive(Debug, Serialize)]
pub struct OrganizationOverview {
    id: String,
    name: String,
    time_zone: String,
    region: String,
    retention_days: i64,
    deletion_due_at: Option<String>,
    role: String,
    workspaces: Vec<WorkspaceSummary>,
    members: Vec<MemberSummary>,
    subscription: Option<SubscriptionSummary>,
    recurring_billing_available: bool,
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    path: String,
    role: String,
    expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeletionResponse {
    status: &'static str,
    deletion_due_at: Option<String>,
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StaffProfile>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let membership = membership_context_including_deleting(&state, &claims.oid).await?;
    let has_workspace = match &membership {
        Some((organization_id, _)) => first_workspace_id(&state, organization_id).await?.is_some(),
        None => false,
    };
    Ok(Json(StaffProfile {
        id: claims.oid,
        name: claims.name,
        email: claims.email,
        has_workspace,
        organization_id: membership.as_ref().map(|(id, _)| id.clone()),
        role: membership.map(|(_, role)| role),
    }))
}

pub async fn get_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoQueue>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    demo::load_queue(&state, &workspace_id).await.map(Json)
}

pub async fn create_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<DemoQueue>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    if let Some((organization_id, _)) =
        membership_context_including_deleting(&state, &claims.oid).await?
    {
        let deletion_due_at: Option<String> =
            sqlx::query_scalar("SELECT deletion_due_at FROM organizations WHERE id = ?")
                .bind(&organization_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| ApiError::internal())?;
        if deletion_due_at.is_some() {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "deletion_pending",
                "Cancel the scheduled firm deletion before you create a workspace.",
            ));
        }
        if let Some(existing) = first_workspace_id(&state, &organization_id).await? {
            return Ok((
                StatusCode::OK,
                Json(demo::load_queue(&state, &existing).await?),
            ));
        }
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
        "INSERT INTO memberships (organization_id, staff_oid, role, accepted_at)
         VALUES (?, ?, 'owner', ?)",
    )
    .bind(&organization_id)
    .bind(&claims.oid)
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
    upsert_staff_user(&state, &claims).await?;
    let workspace_id = require_workspace(&state, &claims.oid).await?;
    create_action_for_workspace(&state, &workspace_id, payload).await
}

pub async fn create_action_in_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    Json(payload): Json<CreateActionRequest>,
) -> Result<(StatusCode, Json<DemoAction>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    require_workspace_access(&state, &claims.oid, &workspace_id).await?;
    create_action_for_workspace(&state, &workspace_id, payload).await
}

async fn create_action_for_workspace(
    state: &AppState,
    workspace_id: &str,
    payload: CreateActionRequest,
) -> Result<(StatusCode, Json<DemoAction>), ApiError> {
    let actor = staff_label(state, workspace_id).await?;
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
    .bind(workspace_id)
    .bind(&action.title)
    .bind(&action.instructions)
    .bind(&action.due_at)
    .bind(now.to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    append_action_audit(&mut tx, workspace_id, &action.id, &actor, now).await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok((StatusCode::CREATED, Json(action)))
}

pub async fn publish_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(action_id): Path<String>,
) -> Result<Json<LinkResponse>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
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
    upsert_staff_user(&state, &claims).await?;
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

pub async fn get_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<OrganizationOverview>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) =
        require_membership_including_deleting(&state, &claims.oid).await?;
    Ok(Json(
        load_organization(&state, &organization_id, &role).await?,
    ))
}

pub async fn create_additional_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAdditionalWorkspaceRequest>,
) -> Result<(StatusCode, Json<DemoQueue>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) = require_membership(&state, &claims.oid).await?;
    require_manager(&role)?;
    validate_label(
        &payload.client_label,
        "Enter a client workspace name in 80 characters or fewer.",
    )?;
    validate_label(
        &payload.client_actor,
        "Enter a client name in 80 characters or fewer.",
    )?;
    let current: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workspaces WHERE namespace = 'real' AND organization_id = ?",
    )
    .bind(&organization_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let limit = workspace_limit(&state, &organization_id).await?;
    if current >= limit {
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "workspace_limit_reached",
            if limit == 1 {
                "A recurring plan is required before you add another client workspace."
            } else {
                "This plan has reached its client workspace limit. Choose another plan or close a workspace."
            },
        ));
    }
    let organization_name: String =
        sqlx::query_scalar("SELECT name FROM organizations WHERE id = ?")
            .bind(&organization_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    let workspace_id = Uuid::now_v7().to_string();
    let now = state.now();
    sqlx::query(
        "INSERT INTO workspaces
         (id, created_at, expires_at, namespace, organization_id, firm_name, client_label, staff_label, client_actor)
         VALUES (?, ?, ?, 'real', ?, ?, ?, ?, ?)",
    )
    .bind(&workspace_id)
    .bind(now.to_rfc3339())
    .bind((now + ChronoDuration::days(36_500)).to_rfc3339())
    .bind(&organization_id)
    .bind(&organization_name)
    .bind(payload.client_label.trim())
    .bind(if claims.name.trim().is_empty() {
        "Workspace member"
    } else {
        claims.name.trim()
    })
    .bind(payload.client_actor.trim())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::CREATED,
        Json(demo::load_queue(&state, &workspace_id).await?),
    ))
}

pub async fn get_workspace_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
) -> Result<Json<DemoQueue>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    require_workspace_access(&state, &claims.oid, &workspace_id).await?;
    demo::load_queue(&state, &workspace_id).await.map(Json)
}

pub async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<OrganizationOverview>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) = require_membership(&state, &claims.oid).await?;
    require_manager(&role)?;
    validate_label(
        &payload.name,
        "Enter a firm name in 80 characters or fewer.",
    )?;
    if !matches!(payload.retention_days, 0 | 30 | 90 | 365) {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_retention",
            "Choose 30, 90, 365 days, or keep records until deletion.",
        ));
    }
    let time_zone = payload.time_zone.trim();
    if time_zone.is_empty() || time_zone.chars().count() > 64 || !time_zone.contains('/') {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_time_zone",
            "Choose a named time zone such as America/New_York.",
        ));
    }
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "UPDATE organizations SET name = ?, time_zone = ?, retention_days = ? WHERE id = ?",
    )
    .bind(payload.name.trim())
    .bind(time_zone)
    .bind(payload.retention_days)
    .bind(&organization_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    sqlx::query("UPDATE workspaces SET firm_name = ? WHERE organization_id = ?")
        .bind(payload.name.trim())
        .bind(&organization_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal())?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(
        load_organization(&state, &organization_id, &role).await?,
    ))
}

pub async fn create_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<InviteMemberRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) = require_membership(&state, &claims.oid).await?;
    require_manager(&role)?;
    if !matches!(payload.role.as_str(), "admin" | "member") {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_role",
            "Choose admin or member access.",
        ));
    }
    let seat_limit = seat_limit(&state, &organization_id).await?;
    let reserved_seats: i64 = sqlx::query_scalar(
        "SELECT
            (SELECT COUNT(*) FROM memberships WHERE organization_id = ?) +
            (SELECT COUNT(*) FROM membership_invitations
             WHERE organization_id = ? AND accepted_at IS NULL AND expires_at > ?)",
    )
    .bind(&organization_id)
    .bind(&organization_id)
    .bind(state.now().to_rfc3339())
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    if reserved_seats >= seat_limit {
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "seat_limit_reached",
            if seat_limit == 1 {
                "A recurring plan is required before you add another staff member."
            } else {
                "This plan has reached its staff seat limit."
            },
        ));
    }
    let token = random_token();
    let expires_at = state.now() + ChronoDuration::days(7);
    sqlx::query(
        "INSERT INTO membership_invitations
         (id, organization_id, token_digest, role, created_by_oid, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(&organization_id)
    .bind(token_digest(&token))
    .bind(&payload.role)
    .bind(&claims.oid)
    .bind(state.now().to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            path: format!("/app/settings#invite={token}"),
            role: payload.role,
            expires_at: expires_at.to_rfc3339(),
        }),
    ))
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AcceptInviteRequest>,
) -> Result<Json<OrganizationOverview>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    if membership_context_including_deleting(&state, &claims.oid)
        .await?
        .is_some()
    {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "membership_exists",
            "This account already belongs to a firm.",
        ));
    }
    let digest = token_digest(payload.token.trim());
    let row = sqlx::query(
        "SELECT i.organization_id, i.role FROM membership_invitations i
         JOIN organizations o ON o.id = i.organization_id
         WHERE i.token_digest = ? AND i.accepted_at IS NULL AND i.expires_at > ?
           AND o.deletion_due_at IS NULL",
    )
    .bind(&digest)
    .bind(state.now().to_rfc3339())
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| {
        ApiError::new(
            StatusCode::GONE,
            "invitation_expired",
            "This staff invitation has expired. Ask the firm owner for a new link.",
        )
    })?;
    let organization_id: String = row.get("organization_id");
    let role: String = row.get("role");
    let seat_limit = seat_limit(&state, &organization_id).await?;
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    let members: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM memberships WHERE organization_id = ?")
            .bind(&organization_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| ApiError::internal())?;
    if members >= seat_limit {
        return Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            "seat_limit_reached",
            "This plan has reached its staff seat limit.",
        ));
    }
    let accepted = sqlx::query(
        "UPDATE membership_invitations SET accepted_by_oid = ?, accepted_at = ?
         WHERE token_digest = ? AND accepted_at IS NULL AND expires_at > ?",
    )
    .bind(&claims.oid)
    .bind(state.now().to_rfc3339())
    .bind(&digest)
    .bind(state.now().to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    if accepted.rows_affected() != 1 {
        return Err(ApiError::new(
            StatusCode::GONE,
            "invitation_expired",
            "This staff invitation has expired. Ask the firm owner for a new link.",
        ));
    }
    sqlx::query(
        "INSERT INTO memberships (organization_id, staff_oid, role, accepted_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&organization_id)
    .bind(&claims.oid)
    .bind(&role)
    .bind(state.now().to_rfc3339())
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::internal())?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(
        load_organization(&state, &organization_id, &role).await?,
    ))
}

pub async fn export_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) = require_membership(&state, &claims.oid).await?;
    require_owner(&role)?;
    let overview = load_organization(&state, &organization_id, &role).await?;
    let actions = sqlx::query(
        "SELECT a.id, a.workspace_id, a.kind, a.title, a.instructions, a.due_at, a.status,
                a.version, a.created_at
         FROM actions a JOIN workspaces w ON w.id = a.workspace_id
         WHERE w.organization_id = ? ORDER BY a.created_at, a.id",
    )
    .bind(&organization_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "id": row.get::<String, _>("id"),
            "workspace_id": row.get::<String, _>("workspace_id"),
            "kind": row.get::<String, _>("kind"),
            "title": row.get::<String, _>("title"),
            "instructions": row.get::<String, _>("instructions"),
            "due_at": row.get::<String, _>("due_at"),
            "status": row.get::<String, _>("status"),
            "version": row.get::<i64, _>("version"),
            "created_at": row.get::<String, _>("created_at"),
        })
    })
    .collect::<Vec<_>>();
    let audit = sqlx::query(
        "SELECT ae.id, ae.workspace_id, ae.action_id, ae.event_name, ae.actor_label,
                ae.decision, ae.occurred_at
         FROM audit_events ae JOIN workspaces w ON w.id = ae.workspace_id
         WHERE w.organization_id = ? ORDER BY ae.occurred_at, ae.id",
    )
    .bind(&organization_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "id": row.get::<String, _>("id"),
            "workspace_id": row.get::<String, _>("workspace_id"),
            "action_id": row.get::<Option<String>, _>("action_id"),
            "event": row.get::<String, _>("event_name"),
            "actor": row.get::<String, _>("actor_label"),
            "decision": row.get::<Option<String>, _>("decision"),
            "occurred_at": row.get::<String, _>("occurred_at"),
        })
    })
    .collect::<Vec<_>>();
    let export = serde_json::json!({
        "format": "client-action-room-export-v1",
        "exported_at": state.now().to_rfc3339(),
        "organization": overview,
        "actions": actions,
        "audit_events": audit,
    });
    let bytes = serde_json::to_vec_pretty(&export).map_err(|_| ApiError::internal())?;
    let checksum = hex::encode(Sha256::digest(&bytes));
    sqlx::query(
        "INSERT INTO organization_exports (id, organization_id, requested_by_oid, checksum_sha256, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(&organization_id)
    .bind(&claims.oid)
    .bind(checksum)
    .bind(state.now().to_rfc3339())
    .bind((state.now() + ChronoDuration::hours(24)).to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let mut response = (StatusCode::OK, bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=client-action-room-export.json"),
    );
    Ok(response)
}

pub async fn request_deletion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DeleteOrganizationRequest>,
) -> Result<(StatusCode, Json<DeletionResponse>), ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) = require_membership(&state, &claims.oid).await?;
    require_owner(&role)?;
    let name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = ?")
        .bind(&organization_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    if payload.confirmation.trim() != name {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "confirmation_mismatch",
            "Type the firm name exactly to schedule deletion.",
        ));
    }
    let now = state.now();
    let due = now + ChronoDuration::days(7);
    sqlx::query(
        "UPDATE organizations SET deletion_requested_at = ?, deletion_due_at = ? WHERE id = ?",
    )
    .bind(now.to_rfc3339())
    .bind(due.to_rfc3339())
    .bind(&organization_id)
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::ACCEPTED,
        Json(DeletionResponse {
            status: "scheduled",
            deletion_due_at: Some(due.to_rfc3339()),
        }),
    ))
}

pub async fn cancel_deletion(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeletionResponse>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, role) =
        require_membership_including_deleting(&state, &claims.oid).await?;
    require_owner(&role)?;
    sqlx::query(
        "UPDATE organizations SET deletion_requested_at = NULL, deletion_due_at = NULL WHERE id = ?",
    )
    .bind(&organization_id)
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(Json(DeletionResponse {
        status: "active",
        deletion_due_at: None,
    }))
}

pub async fn billing_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let claims = state.auth.verify(&headers).await?;
    upsert_staff_user(&state, &claims).await?;
    let (organization_id, _) = require_membership_including_deleting(&state, &claims.oid).await?;
    let subscription = load_subscription(&state, &organization_id).await?;
    Ok(Json(serde_json::json!({
        "available": false,
        "reason": "recurring_billing_not_registered",
        "subscription": subscription,
        "export_available": true
    })))
}

async fn owner_workspace_id(state: &AppState, oid: &str) -> Result<Option<String>, ApiError> {
    sqlx::query_scalar(
        "SELECT w.id FROM workspaces w
         JOIN memberships m ON m.organization_id = w.organization_id
         JOIN organizations o ON o.id = w.organization_id
         WHERE w.namespace = 'real' AND m.staff_oid = ? AND o.deletion_due_at IS NULL
         ORDER BY w.created_at, w.id LIMIT 1",
    )
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())
}

async fn require_membership(state: &AppState, oid: &str) -> Result<(String, String), ApiError> {
    membership_context(state, oid).await?.ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "organization_setup_required",
            "Name your firm and first client workspace to begin.",
        )
    })
}

async fn require_membership_including_deleting(
    state: &AppState,
    oid: &str,
) -> Result<(String, String), ApiError> {
    membership_context_including_deleting(state, oid)
        .await?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "organization_setup_required",
                "Name your firm and first client workspace to begin.",
            )
        })
}

async fn require_workspace_access(
    state: &AppState,
    oid: &str,
    workspace_id: &str,
) -> Result<(String, String), ApiError> {
    let row = sqlx::query(
        "SELECT w.organization_id, m.role FROM workspaces w
         JOIN memberships m ON m.organization_id = w.organization_id
         JOIN organizations o ON o.id = w.organization_id
         WHERE w.id = ? AND w.namespace = 'real' AND m.staff_oid = ?
           AND o.deletion_due_at IS NULL",
    )
    .bind(workspace_id)
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    row.map(|row| (row.get("organization_id"), row.get("role")))
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "workspace_not_found",
                "Choose a workspace that belongs to your firm.",
            )
        })
}

fn require_manager(role: &str) -> Result<(), ApiError> {
    if matches!(role, "owner" | "admin") {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "manager_required",
            "Ask a firm owner or admin to make this change.",
        ))
    }
}

fn require_owner(role: &str) -> Result<(), ApiError> {
    if role == "owner" {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "owner_required",
            "Only a firm owner can make this change.",
        ))
    }
}

async fn load_subscription(
    state: &AppState,
    organization_id: &str,
) -> Result<Option<SubscriptionSummary>, ApiError> {
    let row = sqlx::query(
        "SELECT tier, status, period_end, verified_at FROM subscriptions WHERE organization_id = ?",
    )
    .bind(organization_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(row.map(|row| SubscriptionSummary {
        tier: row.get("tier"),
        status: row.get("status"),
        period_end: row.get("period_end"),
        verified_at: row.get("verified_at"),
    }))
}

async fn subscription_limits(
    state: &AppState,
    organization_id: &str,
) -> Result<Option<(i64, i64)>, ApiError> {
    let row =
        sqlx::query("SELECT tier, status, period_end FROM subscriptions WHERE organization_id = ?")
            .bind(organization_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
    let Some(row) = row else { return Ok(None) };
    let tier: String = row.get("tier");
    let status: String = row.get("status");
    let period_end: Option<String> = row.get("period_end");
    let in_paid_period = matches!(status.as_str(), "active" | "past_due")
        || (status == "cancelled"
            && period_end
                .as_deref()
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .is_some_and(|value| value.with_timezone(&Utc) > state.now()));
    if !in_paid_period {
        return Ok(None);
    }
    Ok(Some(if tier == "studio" { (20, 10) } else { (5, 3) }))
}

async fn workspace_limit(state: &AppState, organization_id: &str) -> Result<i64, ApiError> {
    Ok(subscription_limits(state, organization_id)
        .await?
        .map(|limits| limits.0)
        .unwrap_or(1))
}

async fn seat_limit(state: &AppState, organization_id: &str) -> Result<i64, ApiError> {
    Ok(subscription_limits(state, organization_id)
        .await?
        .map(|limits| limits.1)
        .unwrap_or(1))
}

async fn load_organization(
    state: &AppState,
    organization_id: &str,
    role: &str,
) -> Result<OrganizationOverview, ApiError> {
    let row = sqlx::query(
        "SELECT id, name, time_zone, region, retention_days, deletion_due_at
         FROM organizations WHERE id = ?",
    )
    .bind(organization_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let workspaces = sqlx::query(
        "SELECT w.id, w.client_label, w.client_actor,
                SUM(CASE WHEN a.status = 'open' THEN 1 ELSE 0 END) AS open_actions
         FROM workspaces w LEFT JOIN actions a ON a.workspace_id = w.id
         WHERE w.namespace = 'real' AND w.organization_id = ?
         GROUP BY w.id ORDER BY w.created_at, w.id",
    )
    .bind(organization_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .into_iter()
    .map(|row| WorkspaceSummary {
        id: row.get("id"),
        client_label: row.get("client_label"),
        client_actor: row.get("client_actor"),
        open_actions: row.get("open_actions"),
    })
    .collect();
    let members = sqlx::query(
        "SELECT u.oid, u.display_name, m.role FROM memberships m
         JOIN staff_users u ON u.oid = m.staff_oid
         WHERE m.organization_id = ? ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, m.accepted_at",
    )
    .bind(organization_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .into_iter()
    .map(|row| MemberSummary {
        oid: row.get("oid"),
        display_name: row.get("display_name"),
        role: row.get("role"),
    })
    .collect();
    Ok(OrganizationOverview {
        id: row.get("id"),
        name: row.get("name"),
        time_zone: row.get("time_zone"),
        region: row.get("region"),
        retention_days: row.get("retention_days"),
        deletion_due_at: row.get("deletion_due_at"),
        role: role.to_owned(),
        workspaces,
        members,
        subscription: load_subscription(state, organization_id).await?,
        recurring_billing_available: false,
    })
}

async fn upsert_staff_user(
    state: &AppState,
    claims: &crate::auth::StaffClaims,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO staff_users (oid, display_name, email_snapshot, last_sign_in_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(oid) DO UPDATE SET display_name = excluded.display_name,
             email_snapshot = excluded.email_snapshot, last_sign_in_at = excluded.last_sign_in_at",
    )
    .bind(&claims.oid)
    .bind(claims.name.trim())
    .bind(claims.email.trim())
    .bind(state.now().to_rfc3339())
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(())
}

async fn membership_context(
    state: &AppState,
    oid: &str,
) -> Result<Option<(String, String)>, ApiError> {
    let row = sqlx::query(
        "SELECT m.organization_id, m.role FROM memberships m
         JOIN organizations o ON o.id = m.organization_id
         WHERE m.staff_oid = ? AND o.deletion_due_at IS NULL
         ORDER BY m.accepted_at LIMIT 1",
    )
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(row.map(|row| (row.get("organization_id"), row.get("role"))))
}

async fn membership_context_including_deleting(
    state: &AppState,
    oid: &str,
) -> Result<Option<(String, String)>, ApiError> {
    let row = sqlx::query(
        "SELECT organization_id, role FROM memberships
         WHERE staff_oid = ? ORDER BY accepted_at LIMIT 1",
    )
    .bind(oid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(row.map(|row| (row.get("organization_id"), row.get("role"))))
}

async fn first_workspace_id(
    state: &AppState,
    organization_id: &str,
) -> Result<Option<String>, ApiError> {
    sqlx::query_scalar(
        "SELECT id FROM workspaces WHERE namespace = 'real' AND organization_id = ?
         ORDER BY created_at, id LIMIT 1",
    )
    .bind(organization_id)
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
