use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use shared::{
    models::{CreatePersonalTaskRequest, PersonalTask, UpdatePersonalTaskRequest},
};
use crate::AppState;
use uuid::Uuid;

/// Create a new personal task for the authenticated user
pub async fn create_personal_task(
    State(state): State<AppState>,
    Json(body): Json<CreatePersonalTaskRequest>,
) -> Result<(StatusCode, Json<PersonalTask>), (StatusCode, Json<Value>)> {
    // In real app, extract user_id from JWT claims/headers
    // For now, we'll need this from the request
    let user_id = Uuid::new_v4(); // Placeholder - should come from auth

    let priority = body.priority.unwrap_or_else(|| "medium".to_string());
    let status = "todo".to_string();

    let result = sqlx::query_as::<_, PersonalTask>(
        r#"
        INSERT INTO personal_tasks (user_id, title, description, status, priority, deadline)
        VALUES ($1, $2, $3, $4::task_status, $5::task_priority, $6)
        RETURNING id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&status)
    .bind(&priority)
    .bind(body.deadline)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(task) => Ok((StatusCode::CREATED, Json(task))),
        Err(e) => {
            tracing::error!("Failed to create personal task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create task"})),
            ))
        }
    }
}

/// List personal tasks for the authenticated user
pub async fn list_personal_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<PersonalTask>>, (StatusCode, Json<Value>)> {
    // In real app, extract user_id from JWT
    let user_id = body.user_id; // Placeholder

    let result = sqlx::query_as::<_, PersonalTask>(
        r#"
        SELECT id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
        FROM personal_tasks
        WHERE user_id = $1
        ORDER BY deadline ASC NULLS LAST, created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(tasks) => Ok(Json(tasks)),
        Err(e) => {
            tracing::error!("Failed to list personal tasks: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to list tasks"})),
            ))
        }
    }
}

/// Get a specific personal task
pub async fn get_personal_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<PersonalTask>, (StatusCode, Json<Value>)> {
    let result = sqlx::query_as::<_, PersonalTask>(
        r#"
        SELECT id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
        FROM personal_tasks
        WHERE id = $1
        "#,
    )
    .bind(task_id)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(task) => Ok(Json(task)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        )),
        Err(e) => {
            tracing::error!("Failed to get personal task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get task"})),
            ))
        }
    }
}

/// Update a personal task
pub async fn update_personal_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(body): Json<UpdatePersonalTaskRequest>,
) -> Result<Json<PersonalTask>, (StatusCode, Json<Value>)> {
    // Build dynamic update query
    let mut query = String::from(
        r#"
        UPDATE personal_tasks
        SET updated_at = NOW()
        "#,
    );
    let mut bind_idx = 1;

    if body.title.is_some() {
        query.push_str(&format!(", title = ${}", bind_idx + 1));
        bind_idx += 1;
    }
    if body.description.is_some() {
        query.push_str(&format!(", description = ${}", bind_idx + 1));
        bind_idx += 1;
    }
    if body.status.is_some() {
        query.push_str(&format!(", status = ${}", bind_idx + 1));
        bind_idx += 1;
    }
    if body.priority.is_some() {
        query.push_str(&format!(", priority = ${}", bind_idx + 1));
        bind_idx += 1;
    }
    if body.deadline.is_some() {
        query.push_str(&format!(", deadline = ${}", bind_idx + 1));
        bind_idx += 1;
    }

    query.push_str(&format!(
        r#"
        WHERE id = ${}
        RETURNING id, user_id, title, description, status, priority, deadline, created_at, updated_at
        "#,
        bind_idx + 1
    ));

    // Execute with simplified approach - just update what's provided
    let result = if body.title.is_some() | body.description.is_some() | body.status.is_some()
        | body.priority.is_some() | body.deadline.is_some() {
        sqlx::query_as::<_, PersonalTask>(
            r#"
            UPDATE personal_tasks
            SET 
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                status = COALESCE($3::task_status, status),
                priority = COALESCE($4::task_priority, priority),
                deadline = COALESCE($5, deadline),
                updated_at = NOW()
            WHERE id = $6
            RETURNING id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
            "#,
        )
        .bind(body.title)
        .bind(body.description)
        .bind(body.status)
        .bind(body.priority)
        .bind(body.deadline)
        .bind(task_id)
        .fetch_one(&state.db)
        .await
    } else {
        // No updates, return current task
        sqlx::query_as::<_, PersonalTask>(
            r#"
            SELECT id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
            FROM personal_tasks
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_one(&state.db)
        .await
    };

    match result {
        Ok(task) => Ok(Json(task)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        )),
        Err(e) => {
            tracing::error!("Failed to update personal task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update task"})),
            ))
        }
    }
}

/// Delete a personal task
pub async fn delete_personal_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = sqlx::query("DELETE FROM personal_tasks WHERE id = $1")
        .bind(task_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Task not found"})),
                ))
            } else {
                Ok(Json(json!({"message": "Task deleted successfully"})))
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete personal task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete task"})),
            ))
        }
    }
}

/// Get tasks with deadlines (for calendar)
pub async fn get_personal_tasks_with_deadline(
    State(state): State<AppState>,
) -> Result<Json<Vec<PersonalTask>>, (StatusCode, Json<Value>)> {
    // In real app, extract user_id from JWT
    let user_id = Uuid::new_v4(); // Placeholder

    let result = sqlx::query_as::<_, PersonalTask>(
        r#"
        SELECT id, user_id, title, description, CAST(status AS TEXT) as status, CAST(priority AS TEXT) as priority, deadline, created_at, updated_at
        FROM personal_tasks
        WHERE user_id = $1 AND deadline IS NOT NULL
        ORDER BY deadline ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(tasks) => Ok(Json(tasks)),
        Err(e) => {
            tracing::error!("Failed to list personal tasks with deadline: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to list tasks"})),
            ))
        }
    }
}
