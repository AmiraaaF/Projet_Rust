use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::models::{CreateTaskRequest, Task, UpdateTaskRequest};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 { 1 }
fn default_limit() -> i64 { 100 }

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

// Helper to extract user_id from JWT token
fn extract_user_id_from_token(headers: &HeaderMap, state: &AppState) -> Result<Uuid, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid authorization format"})),
            )
        })?;

    let claims = state.auth.validate_token(token).map_err(|_| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token"})),
        )
    })?;

    Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })
}

pub async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(axum::http::StatusCode, Json<Task>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let new_id = Uuid::new_v4();

    let task = sqlx::query_as::<_, Task>(
        r#"
        INSERT INTO tasks (id, project_id, assignee_id, title, description, status, priority, deadline, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6::task_status, 'todo'::task_status), COALESCE($7::task_priority, 'medium'::task_priority), $8, NOW(), NOW())
        RETURNING id, project_id, assignee_id, title, description, status::text AS status, priority::text AS priority, deadline, created_at, updated_at
        "#,
    )
    .bind(new_id)
    .bind(project_id)
    .bind::<Option<Uuid>>(None)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind::<Option<String>>(None)
    .bind(&payload.priority)
    .bind(&payload.deadline)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create task: {}", e)})),
        )
    })?;

    Ok((axum::http::StatusCode::CREATED, Json(task)))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Task>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let offset = (params.page - 1) * params.limit;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tasks WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to count tasks: {}", e)})),
        )
    })?;

    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, project_id, assignee_id, title, description, status::text AS status, priority::text AS priority, deadline, created_at, updated_at
        FROM tasks
        WHERE project_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(project_id)
    .bind(params.limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to fetch tasks: {}", e)})),
        )
    })?;

    Ok(Json(PaginatedResponse {
        data: tasks,
        total: total.0,
        page: params.page,
        limit: params.limit,
    }))
}

pub async fn get_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, project_id, assignee_id, title, description, status::text AS status, priority::text AS priority, deadline, created_at, updated_at
        FROM tasks
        WHERE id = $1
        "#,
    )
    .bind(task_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        )
    })?;

    Ok(Json(task))
}

pub async fn update_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        UPDATE tasks
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3::task_status, status),
            priority = COALESCE($4::task_priority, priority),
            assignee_id = COALESCE($5, assignee_id),
            deadline = COALESCE($6, deadline),
            updated_at = NOW()
        WHERE id = $7
        RETURNING id, project_id, assignee_id, title, description, status::text AS status, priority::text AS priority, deadline, created_at, updated_at
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.status)
    .bind(&payload.priority)
    .bind(&payload.assignee_id)
    .bind(&payload.deadline)
    .bind(task_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        )
    })?;

    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let result = sqlx::query(
        "DELETE FROM tasks WHERE id = $1"
    )
    .bind(task_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        ));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
