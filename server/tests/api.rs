use std::path::PathBuf;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::Response,
};
use chrono::{TimeZone, Utc};
use client_action_room_api::{app, state::AppState};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, Row};
use tower::ServiceExt;

async fn state() -> AppState {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    AppState::new(
        "api-test",
        pool,
        Some(Utc.with_ymd_and_hms(2026, 8, 28, 14, 0, 0).unwrap()),
        PathBuf::from("../dist"),
    )
}

async fn send(
    router: axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", "localhost:4173")
        .header("x-forwarded-for", "198.51.100.20");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if body.is_some() {
        builder = builder
            .header(header::CONTENT_TYPE, "application/json")
            .header("idempotency-key", "api-test-request-0001");
    }
    router
        .oneshot(
            builder
                .body(Body::from(
                    body.map(|value| value.to_string()).unwrap_or_default(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send_auth(
    router: axum::Router,
    method: &str,
    uri: &str,
    oid: &str,
    body: Option<Value>,
) -> Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", "localhost:4173")
        .header(header::AUTHORIZATION, format!("Bearer test:{oid}"));
    if body.is_some() {
        builder = builder
            .header(header::CONTENT_TYPE, "application/json")
            .header("idempotency-key", format!("request-{oid}-0001"));
    }
    router
        .oneshot(
            builder
                .body(Body::from(
                    body.map(|value| value.to_string()).unwrap_or_default(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn cookie(response: &Response, name: &str) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with(&format!("{name}=")))
        .and_then(|value| value.split(';').next())
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn workspaces_are_isolated_and_reset_reseeds() {
    let state = state().await;
    let router = app(state.clone());
    let first = send(
        router.clone(),
        "POST",
        "/api/v1/demo/sessions",
        None,
        Some(json!({})),
    )
    .await;
    let first_cookie = cookie(&first, "car_demo");
    let first_body = json_body(first).await;
    let second = send(
        router.clone(),
        "POST",
        "/api/v1/demo/sessions",
        None,
        Some(json!({})),
    )
    .await;
    let second_cookie = cookie(&second, "car_demo");
    assert_ne!(first_cookie, second_cookie);
    assert_eq!(first_body["actions"].as_array().unwrap().len(), 4);

    let approval_id = first_body["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "approval")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let published = send(
        router.clone(),
        "POST",
        &format!("/api/v1/demo/actions/{approval_id}/publish"),
        Some(&first_cookie),
        Some(json!({})),
    )
    .await;
    assert_eq!(published.status(), StatusCode::OK);

    let untouched = send(
        router.clone(),
        "GET",
        "/api/v1/demo/queue",
        Some(&second_cookie),
        None,
    )
    .await;
    let untouched = json_body(untouched).await;
    assert_eq!(untouched["audit"].as_array().unwrap().len(), 2);

    let reset = send(
        router,
        "POST",
        "/api/v1/demo/session/reset",
        Some(&first_cookie),
        Some(json!({})),
    )
    .await;
    let reset_body = json_body(reset).await;
    assert_eq!(reset_body["actions"].as_array().unwrap().len(), 4);
    assert_eq!(reset_body["audit"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn grant_exchange_submission_and_audit_are_scoped_and_idempotent() {
    let state = state().await;
    let router = app(state.clone());
    let created = send(
        router.clone(),
        "POST",
        "/api/v1/demo/sessions",
        None,
        Some(json!({})),
    )
    .await;
    let demo_cookie = cookie(&created, "car_demo");
    let created_body = json_body(created).await;
    let approval_id = created_body["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "approval")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let published = send(
        router.clone(),
        "POST",
        &format!("/api/v1/demo/actions/{approval_id}/publish"),
        Some(&demo_cookie),
        Some(json!({})),
    )
    .await;
    let published_body = json_body(published).await;
    let token = published_body["path"]
        .as_str()
        .unwrap()
        .split("access=")
        .nth(1)
        .unwrap();

    let stored: String = sqlx::query("SELECT token_digest FROM client_grants LIMIT 1")
        .fetch_one(&state.pool)
        .await
        .unwrap()
        .get("token_digest");
    assert!(stored.starts_with("v1:"));
    assert!(!stored.contains(token));

    let exchanged = send(
        router.clone(),
        "POST",
        "/api/v1/client-links/exchange",
        None,
        Some(json!({ "token": token })),
    )
    .await;
    assert_eq!(exchanged.status(), StatusCode::OK);
    let client_cookie = cookie(&exchanged, "car_client");
    let client_view = send(
        router.clone(),
        "GET",
        "/api/v1/client/actions",
        Some(&client_cookie),
        None,
    )
    .await;
    let client_body = json_body(client_view).await;
    assert_eq!(client_body["action"]["id"], approval_id);

    let payload = json!({
        "actor_label": "Maya Chen",
        "decision": "approved",
        "comment": "Ready to print"
    });
    let first = send(
        router.clone(),
        "POST",
        &format!("/api/v1/client/actions/{approval_id}/submissions"),
        Some(&client_cookie),
        Some(payload.clone()),
    )
    .await;
    let first_body = json_body(first).await;
    assert_eq!(first_body["replayed"], false);
    assert_eq!(first_body["occurred_at"], "2026-08-28T14:00:00+00:00");
    let replay = send(
        router.clone(),
        "POST",
        &format!("/api/v1/client/actions/{approval_id}/submissions"),
        Some(&client_cookie),
        Some(payload),
    )
    .await;
    assert_eq!(json_body(replay).await["replayed"], true);

    let queue = send(
        router,
        "GET",
        "/api/v1/demo/queue",
        Some(&demo_cookie),
        None,
    )
    .await;
    let queue = json_body(queue).await;
    let recorded = queue["audit"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["event_name"] == "client_decision_recorded")
        .unwrap();
    assert_eq!(recorded["actor_label"], "Maya Chen");
    assert_eq!(recorded["decision"], "approved");
    assert_eq!(recorded["occurred_at"], "2026-08-28T14:00:00+00:00");
}

#[tokio::test]
async fn expired_grant_cannot_read_or_submit() {
    let state = state().await;
    let router = app(state);
    let created = send(
        router.clone(),
        "POST",
        "/api/v1/demo/sessions",
        None,
        Some(json!({})),
    )
    .await;
    let demo_cookie = cookie(&created, "car_demo");
    let expired = send(
        router.clone(),
        "POST",
        "/api/v1/demo/client-links/expired",
        Some(&demo_cookie),
        Some(json!({})),
    )
    .await;
    let body = json_body(expired).await;
    let token = body["path"]
        .as_str()
        .unwrap()
        .split("access=")
        .nth(1)
        .unwrap();
    let exchange = send(
        router.clone(),
        "POST",
        "/api/v1/client-links/exchange",
        None,
        Some(json!({ "token": token })),
    )
    .await;
    assert_eq!(exchange.status(), StatusCode::GONE);
    let read = send(router, "GET", "/api/v1/client/actions", None, None).await;
    assert_eq!(read.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(read.headers()[header::WWW_AUTHENTICATE], "Bearer");
}

#[tokio::test]
async fn staff_identity_rejects_missing_and_untrusted_tokens() {
    let router = app(state().await);
    let missing = send(router.clone(), "GET", "/api/v1/me", None, None).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(missing.headers()[header::WWW_AUTHENTICATE], "Bearer");

    let forged = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header(header::AUTHORIZATION, "Bearer not-a-jwt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn staff_workspace_is_stable_for_oid_and_isolated_from_another_oid() {
    let state = state().await;
    let router = app(state.clone());
    let absent = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/workspace",
        "oid-one",
        None,
    )
    .await;
    assert_eq!(absent.status(), StatusCode::NOT_FOUND);

    let created = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspace",
        "oid-one",
        Some(json!({
            "firm_name": "River & Pine",
            "client_label": "March launch",
            "client_actor": "Ari Kim"
        })),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let created = json_body(created).await;
    assert_eq!(created["namespace"], "real");
    assert_eq!(created["firm"], "River & Pine");
    assert_eq!(created["workspace"], "March launch");
    assert_eq!(created["actions"].as_array().unwrap().len(), 0);

    let approval = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/actions",
        "oid-one",
        Some(json!({
            "title": "Approve the launch copy",
            "instructions": "Check the three headings.",
            "due_at": "2026-08-29T14:00:00Z"
        })),
    )
    .await;
    assert_eq!(approval.status(), StatusCode::CREATED);

    let same = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/workspace",
        "oid-one",
        None,
    )
    .await;
    assert_eq!(
        json_body(same).await["actions"].as_array().unwrap().len(),
        1
    );

    let other = send_auth(router, "GET", "/api/v1/staff/workspace", "oid-two", None).await;
    assert_eq!(other.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reversible_migration_removes_demo_schema() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations/202608280001_demo.up.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202608280002_action_types.up.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609050003_real_workspaces.up.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609060004_demo_delivery_queue.up.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609060005_accounts_persistence.up.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609060005_accounts_persistence.down.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609060004_demo_delivery_queue.down.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202609050003_real_workspaces.down.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/202608280002_action_types.down.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!("../migrations/202608280001_demo.down.sql"))
        .execute(&pool)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name LIKE 'demo_%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn organization_membership_limits_export_and_delete_recovery_are_enforced() {
    let state = state().await;
    let router = app(state.clone());
    let created = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspace",
        "m2-owner",
        Some(json!({
            "firm_name": "Juniper Works",
            "client_label": "Harbor launch",
            "client_actor": "Sam Lee"
        })),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);

    let overview = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization",
        "m2-owner",
        None,
    )
    .await;
    assert_eq!(overview.status(), StatusCode::OK);
    let overview = json_body(overview).await;
    let organization_id = overview["id"].as_str().unwrap().to_owned();
    assert_eq!(overview["members"][0]["role"], "owner");
    assert_eq!(overview["workspaces"].as_array().unwrap().len(), 1);

    let unpaid_second = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspaces",
        "m2-owner",
        Some(json!({"client_label": "Second client", "client_actor": "Robin"})),
    )
    .await;
    assert_eq!(unpaid_second.status(), StatusCode::PAYMENT_REQUIRED);

    sqlx::query(
        "INSERT INTO subscriptions
         (organization_id, provider_reference, tier, status, period_end, verified_at, provider_event_id)
         VALUES (?, 'fixture-subscription', 'starter', 'active', ?, ?, 'fixture-event')",
    )
    .bind(&organization_id)
    .bind("2026-09-28T14:00:00+00:00")
    .bind("2026-08-28T14:00:00+00:00")
    .execute(&state.pool)
    .await
    .unwrap();

    let paid_second = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspaces",
        "m2-owner",
        Some(json!({"client_label": "Second client", "client_actor": "Robin"})),
    )
    .await;
    assert_eq!(paid_second.status(), StatusCode::CREATED);

    let invitation = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/members/invitations",
        "m2-owner",
        Some(json!({"role": "member"})),
    )
    .await;
    assert_eq!(invitation.status(), StatusCode::CREATED);
    let invitation = json_body(invitation).await;
    let token = invitation["path"]
        .as_str()
        .unwrap()
        .split("invite=")
        .nth(1)
        .unwrap();
    let accepted = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/members/invitations/accept",
        "m2-member",
        Some(json!({"token": token})),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::OK);
    assert_eq!(json_body(accepted).await["role"], "member");
    let member_read = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization",
        "m2-member",
        None,
    )
    .await;
    assert_eq!(member_read.status(), StatusCode::OK);
    let first_workspace_id = overview["workspaces"][0]["id"].as_str().unwrap();
    let scoped_action = send_auth(
        router.clone(),
        "POST",
        &format!("/api/v1/staff/workspaces/{first_workspace_id}/actions"),
        "m2-owner",
        Some(json!({
            "title": "Approve the launch note",
            "instructions": "Read the final note and approve it.",
            "due_at": "2026-08-30T14:00:00+00:00"
        })),
    )
    .await;
    assert_eq!(scoped_action.status(), StatusCode::CREATED);
    let member_export = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization/export",
        "m2-member",
        None,
    )
    .await;
    assert_eq!(member_export.status(), StatusCode::FORBIDDEN);
    let reserved_invitation = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/members/invitations",
        "m2-owner",
        Some(json!({"role": "admin"})),
    )
    .await;
    assert_eq!(reserved_invitation.status(), StatusCode::CREATED);
    let over_seat_limit = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/members/invitations",
        "m2-owner",
        Some(json!({"role": "member"})),
    )
    .await;
    assert_eq!(over_seat_limit.status(), StatusCode::PAYMENT_REQUIRED);
    let outsider = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization",
        "m2-outsider",
        None,
    )
    .await;
    assert_eq!(outsider.status(), StatusCode::NOT_FOUND);
    let outsider_workspace = send_auth(
        router.clone(),
        "GET",
        &format!("/api/v1/staff/workspaces/{first_workspace_id}"),
        "m2-outsider",
        None,
    )
    .await;
    assert_eq!(outsider_workspace.status(), StatusCode::NOT_FOUND);
    let outsider_action = send_auth(
        router.clone(),
        "POST",
        &format!("/api/v1/staff/workspaces/{first_workspace_id}/actions"),
        "m2-outsider",
        Some(json!({
            "title": "Cross-tenant write",
            "instructions": "This must never be created.",
            "due_at": "2026-08-30T14:00:00+00:00"
        })),
    )
    .await;
    assert_eq!(outsider_action.status(), StatusCode::NOT_FOUND);

    let settings = send_auth(
        router.clone(),
        "PATCH",
        "/api/v1/staff/organization",
        "m2-owner",
        Some(json!({
            "name": "Juniper Works Ltd",
            "time_zone": "Europe/London",
            "retention_days": 365
        })),
    )
    .await;
    assert_eq!(settings.status(), StatusCode::OK);
    assert_eq!(json_body(settings).await["retention_days"], 365);

    sqlx::query("UPDATE subscriptions SET status = 'cancelled' WHERE organization_id = ?")
        .bind(&organization_id)
        .execute(&state.pool)
        .await
        .unwrap();
    let export = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization/export",
        "m2-owner",
        None,
    )
    .await;
    assert_eq!(export.status(), StatusCode::OK);
    assert_eq!(
        export.headers()[header::CONTENT_DISPOSITION],
        "attachment; filename=client-action-room-export.json"
    );
    let export_body = json_body(export).await;
    assert_eq!(
        export_body["organization"]["workspaces"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let wrong_delete = send_auth(
        router.clone(),
        "DELETE",
        "/api/v1/staff/organization",
        "m2-owner",
        Some(json!({"confirmation": "wrong"})),
    )
    .await;
    assert_eq!(wrong_delete.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let deletion = send_auth(
        router.clone(),
        "DELETE",
        "/api/v1/staff/organization",
        "m2-owner",
        Some(json!({"confirmation": "Juniper Works Ltd"})),
    )
    .await;
    assert_eq!(deletion.status(), StatusCode::ACCEPTED);
    let scheduled_organization = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/organization",
        "m2-owner",
        None,
    )
    .await;
    assert_eq!(scheduled_organization.status(), StatusCode::OK);
    assert!(json_body(scheduled_organization).await["deletion_due_at"]
        .as_str()
        .is_some());
    let blocked_workspace = send_auth(
        router.clone(),
        "GET",
        "/api/v1/staff/workspace",
        "m2-owner",
        None,
    )
    .await;
    assert_eq!(blocked_workspace.status(), StatusCode::NOT_FOUND);
    let blocked_recreation = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspace",
        "m2-owner",
        Some(json!({
            "firm_name": "Replacement firm",
            "client_label": "Replacement client",
            "client_actor": "Replacement actor"
        })),
    )
    .await;
    assert_eq!(blocked_recreation.status(), StatusCode::CONFLICT);
    let cancelled = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/organization/deletion/cancel",
        "m2-owner",
        Some(json!({})),
    )
    .await;
    assert_eq!(cancelled.status(), StatusCode::OK);
    let restored = send_auth(
        router,
        "GET",
        "/api/v1/staff/organization",
        "m2-owner",
        None,
    )
    .await;
    assert_eq!(restored.status(), StatusCode::OK);
}

#[tokio::test]
async fn scheduled_organization_deletion_purges_owned_records_at_seven_days() {
    let state = state().await;
    let router = app(state.clone());
    let created = send_auth(
        router.clone(),
        "POST",
        "/api/v1/staff/workspace",
        "delete-owner",
        Some(json!({
            "firm_name": "Delete Test Firm",
            "client_label": "Private client",
            "client_actor": "Taylor"
        })),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let deletion = send_auth(
        router.clone(),
        "DELETE",
        "/api/v1/staff/organization",
        "delete-owner",
        Some(json!({"confirmation": "Delete Test Firm"})),
    )
    .await;
    assert_eq!(deletion.status(), StatusCode::ACCEPTED);

    assert!(state.set_test_clock(Utc.with_ymd_and_hms(2026, 9, 4, 13, 59, 59).unwrap()));
    assert_eq!(state.purge_expired().await.unwrap(), 0);
    let before_due: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    assert_eq!(before_due, 1);

    assert!(state.set_test_clock(Utc.with_ymd_and_hms(2026, 9, 4, 14, 0, 0).unwrap()));
    assert_eq!(state.purge_expired().await.unwrap(), 2);
    for table in ["organizations", "workspaces", "memberships", "staff_users"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&state.pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "{table} should be empty after deletion");
    }
    let deleted = send_auth(
        router,
        "GET",
        "/api/v1/staff/organization",
        "delete-owner",
        None,
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn session_survives_a_request_served_by_a_second_app_instance() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("shared.sqlite3");
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let first_pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!().run(&first_pool).await.unwrap();
    let second_pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    let clock = Some(Utc.with_ymd_and_hms(2026, 8, 28, 14, 0, 0).unwrap());
    let first = app(AppState::new(
        "first",
        first_pool,
        clock,
        PathBuf::from("../dist"),
    ));
    let second = app(AppState::new(
        "second",
        second_pool,
        clock,
        PathBuf::from("../dist"),
    ));

    let created = send(
        first,
        "POST",
        "/api/v1/demo/session/ensure",
        None,
        Some(json!({})),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let session_cookie = cookie(&created, "car_demo");
    let queue = send(
        second,
        "GET",
        "/api/v1/demo/queue",
        Some(&session_cookie),
        None,
    )
    .await;
    assert_eq!(queue.status(), StatusCode::OK);
    assert_eq!(
        json_body(queue).await["actions"].as_array().unwrap().len(),
        4
    );
}

#[tokio::test]
async fn successful_mutation_can_be_snapshotted_and_restored() {
    let directory = tempfile::tempdir().unwrap();
    let live_path = directory.path().join("live.sqlite3");
    let snapshot_path = directory.path().join("durable.sqlite3");
    let url = format!("sqlite://{}?mode=rwc", live_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let state = AppState::new_with_persistence(
        "snapshot",
        pool,
        Some(Utc.with_ymd_and_hms(2026, 8, 28, 14, 0, 0).unwrap()),
        PathBuf::from("../dist"),
        Some(live_path),
        Some(snapshot_path.clone()),
    );
    let response = send_auth(
        app(state.clone()),
        "POST",
        "/api/v1/staff/workspace",
        "durable-oid",
        Some(json!({
            "firm_name": "Durable Firm",
            "client_label": "Durable Client",
            "client_actor": "Dana"
        })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    state.persist_snapshot().await.unwrap();
    assert!(snapshot_path.exists());

    let restored_url = format!("sqlite://{}?mode=ro", snapshot_path.display());
    let restored = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&restored_url)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE owner_oid = 'durable-oid'")
            .fetch_one(&restored)
            .await
            .unwrap();
    assert_eq!(count, 1);
}
