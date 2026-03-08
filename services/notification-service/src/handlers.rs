use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// ─────────────────────────────────────────────────────────────────────────────
//  QUERY PARAMS
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct NotificationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
    pub notification_type: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  LIST NOTIFICATIONS 
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_notifications(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<NotificationQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;


    let mut where_clauses = vec!["user_id = $1".to_string()];
    let mut param_idx = 2usize;

    if let Some(ref status) = q.status {
        where_clauses.push(format!("CAST(status AS TEXT) = ${}", param_idx));
        param_idx += 1;
    }
    if let Some(ref ntype) = q.notification_type {
        where_clauses.push(format!("CAST(notification_type AS TEXT) = ${}", param_idx));
        param_idx += 1;
    }

    let where_str = where_clauses.join(" AND ");


    let unread_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND CAST(status AS TEXT) != 'read'"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);


    let count_sql = format!("SELECT COUNT(*) FROM notifications WHERE {}", where_str);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(user_id);
    if let Some(ref status) = q.status {
        count_query = count_query.bind(status);
    }
    if let Some(ref ntype) = q.notification_type {
        count_query = count_query.bind(ntype);
    }
    let total: i64 = count_query.fetch_one(&state.db).await.unwrap_or(0);

    let main_sql = format!(
        r#"
        SELECT id, user_id, title, message,
               CAST(notification_type AS TEXT),
               CAST(status AS TEXT),
               read_at, created_at, updated_at
        FROM notifications
        WHERE {}
        ORDER BY created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_str, limit, offset
    );

    let mut main_query = sqlx::query(&main_sql).bind(user_id);
    if let Some(ref status) = q.status {
        main_query = main_query.bind(status);
    }
    if let Some(ref ntype) = q.notification_type {
        main_query = main_query.bind(ntype);
    }

    match main_query.fetch_all(&state.db).await {
        Ok(rows) => {
            let data: Vec<Value> = rows.iter().map(|r| {
                json!({
                    "id":                r.get::<Uuid, _>(0),
                    "user_id":           r.get::<Uuid, _>(1),
                    "title":             r.get::<String, _>(2),
                    "message":           r.get::<String, _>(3),
                    "notification_type": r.get::<String, _>(4),
                    "status":            r.get::<String, _>(5),
                    "read_at":           r.get::<Option<chrono::DateTime<Utc>>, _>(6),
                    "created_at":        r.get::<chrono::DateTime<Utc>, _>(7),
                    "updated_at":        r.get::<chrono::DateTime<Utc>, _>(8),
                })
            }).collect();

            Ok(Json(json!({
                "data":         data,
                "page":         page,
                "limit":        limit,
                "total":        total,
                "unread_count": unread_count,
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  GET SINGLE NOTIFICATION
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_notification(
    State(state): State<AppState>,
    Path(notif_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        SELECT id, user_id, title, message,
               CAST(notification_type AS TEXT),
               CAST(status AS TEXT),
               read_at, created_at, updated_at
        FROM notifications WHERE id = $1
        "#,
    )
    .bind(notif_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => Ok(Json(json!({
            "id":                r.get::<Uuid, _>(0),
            "user_id":           r.get::<Uuid, _>(1),
            "title":             r.get::<String, _>(2),
            "message":           r.get::<String, _>(3),
            "notification_type": r.get::<String, _>(4),
            "status":            r.get::<String, _>(5),
            "read_at":           r.get::<Option<chrono::DateTime<Utc>>, _>(6),
            "created_at":        r.get::<chrono::DateTime<Utc>, _>(7),
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Notification not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  CREATE NOTIFICATION 
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateNotifBody {
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: Option<String>,
}

pub async fn create_notification(
    State(state): State<AppState>,
    Json(body): Json<CreateNotifBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let notif_type = body.notification_type.as_deref().unwrap_or("in_app");
    let valid_types = ["email", "in_app", "system"];
    if !valid_types.contains(&notif_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid notification_type '{}'. Must be: email, in_app, system", notif_type)})),
        ));
    }

    match sqlx::query(
        r#"
        INSERT INTO notifications
            (user_id, title, message, notification_type, status, created_at, updated_at)
        VALUES ($1, $2, $3, ($4)::notification_type, 'sent'::notification_status, NOW(), NOW())
        RETURNING id, user_id, title, message,
                  CAST(notification_type AS TEXT),
                  CAST(status AS TEXT),
                  created_at
        "#,
    )
    .bind(body.user_id)
    .bind(&body.title)
    .bind(&body.message)
    .bind(notif_type)
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => Ok((
            StatusCode::CREATED,
            Json(json!({
                "id":                r.get::<Uuid, _>(0),
                "user_id":           r.get::<Uuid, _>(1),
                "title":             r.get::<String, _>(2),
                "message":           r.get::<String, _>(3),
                "notification_type": r.get::<String, _>(4),
                "status":            r.get::<String, _>(5),
                "created_at":        r.get::<chrono::DateTime<Utc>, _>(6),
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  MARK ONE AS READ
// ─────────────────────────────────────────────────────────────────────────────

pub async fn mark_read(
    State(state): State<AppState>,
    Path(notif_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        UPDATE notifications
        SET status = 'read'::notification_status, read_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING id, CAST(status AS TEXT), read_at
        "#,
    )
    .bind(notif_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => Ok(Json(json!({
            "id":      r.get::<Uuid, _>(0),
            "status":  r.get::<String, _>(1),
            "read_at": r.get::<Option<chrono::DateTime<Utc>>, _>(2),
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Notification not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  MARK ALL AS READ (for a user)
// ─────────────────────────────────────────────────────────────────────────────

pub async fn mark_all_read(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query_scalar::<_, i64>(
        r#"
        WITH updated AS (
            UPDATE notifications
            SET status = 'read'::notification_status, read_at = NOW(), updated_at = NOW()
            WHERE user_id = $1 AND CAST(status AS TEXT) != 'read'
            RETURNING 1
        )
        SELECT COUNT(*) FROM updated
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(count) => Ok(Json(json!({
            "message": format!("{} notifications marked as read", count),
            "count":   count,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  DELETE ONE NOTIFICATION
// ─────────────────────────────────────────────────────────────────────────────

pub async fn delete_notification(
    State(state): State<AppState>,
    Path(notif_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM notifications WHERE id = $1 RETURNING id",
    )
    .bind(notif_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => Ok(Json(json!({"message": "Notification deleted", "id": id}))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Notification not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  DELETE ALL READ NOTIFICATIONS 
// ─────────────────────────────────────────────────────────────────────────────

pub async fn delete_all_read(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query_scalar::<_, i64>(
        r#"
        WITH deleted AS (
            DELETE FROM notifications
            WHERE user_id = $1 AND CAST(status AS TEXT) = 'read'
            RETURNING 1
        )
        SELECT COUNT(*) FROM deleted
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(count) => Ok(Json(json!({
            "message": format!("{} read notifications deleted", count),
            "count":   count,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  STATS (unread count, totals by type)
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_stats(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let unread: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND CAST(status AS TEXT) != 'read'"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // total
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // by type
    let by_type = sqlx::query(
        r#"
        SELECT CAST(notification_type AS TEXT), COUNT(*)
        FROM notifications WHERE user_id = $1
        GROUP BY notification_type
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut types_map = serde_json::Map::new();
    for row in &by_type {
        let t: String = row.get(0);
        let c: i64    = row.get(1);
        types_map.insert(t, json!(c));
    }

    Ok(Json(json!({
        "user_id":      user_id,
        "unread_count": unread,
        "total":        total,
        "by_type":      types_map,
    })))
}


#[derive(Deserialize)]
pub struct EventBody {
    pub user_id: Uuid,
    pub event_type: String,
    pub payload: Option<Value>,
}

pub async fn send_event(
    State(state): State<AppState>,
    Json(body): Json<EventBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let payload = body.payload.as_ref().unwrap_or(&Value::Null);

    let (title, message, notif_type) = match body.event_type.as_str() {
        "plan_upgraded" => {
            let plan = payload.get("plan").and_then(|v| v.as_str()).unwrap_or("unknown");
            (
                "🎉 Subscription upgraded".to_string(),
                format!("Your plan has been upgraded to {}. Enjoy your new limits!", plan),
                "system",
            )
        }
        "plan_cancelled" => (
            "📋 Subscription cancelled".to_string(),
            "Your subscription has been cancelled and reverted to the Free plan.".to_string(),
            "system",
        ),
        "invoice_created" => {
            let amount = payload.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let currency = payload.get("currency").and_then(|v| v.as_str()).unwrap_or("USD");
            (
                "🧾 New invoice generated".to_string(),
                format!("An invoice of ${:.2} {} has been issued for your subscription.", amount, currency),
                "email",
            )
        }
        "invoice_paid" => {
            let amount = payload.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            (
                "✅ Payment confirmed".to_string(),
                format!("Your payment of ${:.2} has been confirmed. Thank you!", amount),
                "email",
            )
        }
        "project_created" => {
            let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("a project");
            (
                "📁 New project created".to_string(),
                format!("Project \"{}\" has been successfully created.", name),
                "in_app",
            )
        }
        "task_assigned" => {
            let task = payload.get("title").and_then(|v| v.as_str()).unwrap_or("a task");
            let project = payload.get("project").and_then(|v| v.as_str()).unwrap_or("your project");
            (
                "✅ Task assigned to you".to_string(),
                format!("You have been assigned to task \"{}\" in project \"{}\".", task, project),
                "in_app",
            )
        }
        "member_added" => {
            let project = payload.get("project").and_then(|v| v.as_str()).unwrap_or("a project");
            (
                "👥 Added to a project".to_string(),
                format!("You have been added as a member to project \"{}\".", project),
                "in_app",
            )
        }
        "quota_warning" => {
            let resource = payload.get("resource").and_then(|v| v.as_str()).unwrap_or("resource");
            let pct = payload.get("percent").and_then(|v| v.as_u64()).unwrap_or(80);
            (
                "⚠️ Quota warning".to_string(),
                format!("You have used {}% of your {} quota. Consider upgrading your plan.", pct, resource),
                "system",
            )
        }
        other => (
            format!("🔔 Event: {}", other),
            payload.get("message").and_then(|v| v.as_str()).unwrap_or("A new event occurred.").to_string(),
            "in_app",
        ),
    };

    match sqlx::query(
        r#"
        INSERT INTO notifications
            (user_id, title, message, notification_type, status, created_at, updated_at)
        VALUES ($1, $2, $3, ($4)::notification_type, 'sent'::notification_status, NOW(), NOW())
        RETURNING id, CAST(notification_type AS TEXT), CAST(status AS TEXT), created_at
        "#,
    )
    .bind(body.user_id)
    .bind(&title)
    .bind(&message)
    .bind(notif_type)
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => Ok((
            StatusCode::CREATED,
            Json(json!({
                "id":                r.get::<Uuid, _>(0),
                "user_id":           body.user_id,
                "title":             title,
                "message":           message,
                "notification_type": r.get::<String, _>(1),
                "status":            r.get::<String, _>(2),
                "created_at":        r.get::<chrono::DateTime<Utc>, _>(3),
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}