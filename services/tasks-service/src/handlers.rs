use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateTaskRequest, TaskFilters};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub async fn create_task(State(state): State<AppState>,Json(payload): Json<CreateTaskRequest>,) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        INSERT INTO tasks (project_id, assignee_id, title, description, status, priority, deadline)
        VALUES ($1, $2, $3, $4, COALESCE($5::task_status, 'todo'::task_status), COALESCE($6::task_priority, 'medium'::task_priority), $7)
        RETURNING id, project_id, assignee_id, title, description,
                  CAST(status AS TEXT), CAST(priority AS TEXT), deadline, created_at, updated_at
        "#,
    )
    .bind(payload.project_id)
    .bind(payload.assignee_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.status.as_deref())
    .bind(payload.priority.as_deref())
    .bind(payload.deadline)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => Ok((StatusCode::CREATED, Json(task_row_to_json(&row)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let search_pattern = filters.search.as_deref()
        .map(|s| format!("%{}%", s));

    match sqlx::query(
        r#"
        SELECT id, project_id, assignee_id, title, description,
               CAST(status AS TEXT), CAST(priority AS TEXT), deadline, created_at, updated_at
        FROM tasks
        WHERE ($1::uuid IS NULL OR project_id = $1)
          AND ($2::uuid IS NULL OR assignee_id = $2)
          AND ($3::text IS NULL OR CAST(status AS TEXT) = $3)
          AND ($4::text IS NULL OR CAST(priority AS TEXT) = $4)
          AND ($5::text IS NULL OR title ILIKE $5)
        ORDER BY created_at DESC
        "#,
    )
    .bind(filters.project_id)
    .bind(filters.assignee_id)
    .bind(filters.status.as_deref())
    .bind(filters.priority.as_deref())
    .bind(search_pattern.as_deref())
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let data: Vec<Value> = rows.iter().map(|r| task_row_to_json(r)).collect();
            Ok(Json(json!(data)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn task_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        SELECT CAST(status AS TEXT), COUNT(*)::int as count
        FROM tasks
        GROUP BY status
        "#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            use sqlx::Row;
            let mut stats = json!({
                "todo": 0,
                "in_progress": 0,
                "done": 0,
            });
            for row in &rows {
                let status: String = row.get(0);
                let count: i32 = row.get(1);
                stats[status] = json!(count);
            }
            Ok(Json(stats))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        SELECT id, project_id, assignee_id, title, description,
               CAST(status AS TEXT), CAST(priority AS TEXT), deadline, created_at, updated_at
        FROM tasks WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Ok(Json(task_row_to_json(&row))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Task {} not found", id)})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// pub async fn update_task(
//     State(state): State<AppState>,
//     Path(id): Path<Uuid>,
//     Json(payload): Json<UpdateTaskRequest>,
// ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
//     match sqlx::query(
//         r#"
//         UPDATE tasks SET
//             assignee_id = COALESCE($2, assignee_id),
//             title       = COALESCE($3, title),
//             description = COALESCE($4, description),
//             status      = COALESCE($5::task_status, status),
//             priority    = COALESCE($6::task_priority, priority),
//             deadline    = COALESCE($7, deadline),
//             updated_at  = NOW()
//         WHERE id = $1
//         RETURNING id, project_id, assignee_id, title, description,
//                   CAST(status AS TEXT), CAST(priority AS TEXT), deadline, created_at, updated_at
//         "#,
//     )
//     .bind(id)
//     .bind(payload.assignee_id)
//     .bind(&payload.title)
//     .bind(&payload.description)
//     .bind(payload.status.as_deref())
//     .bind(payload.priority.as_deref())
//     .bind(payload.deadline)
//     .fetch_optional(&state.db)
//     .await
//     {
//         Ok(Some(row)) => Ok(Json(task_row_to_json(&row))),
//         Ok(None) => Err((
//             StatusCode::NOT_FOUND,
//             Json(json!({"error": format!("Task {} not found", id)})),
//         )),
//         Err(e) => Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": e.to_string()})),
//         )),
//     }
// }

pub async fn mark_task_done(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        UPDATE tasks SET
            status     = 'done'::task_status,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, project_id, assignee_id, title, description,
                  CAST(status AS TEXT), CAST(priority AS TEXT), deadline, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Ok(Json(task_row_to_json(&row))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Task {} not found", id)})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match sqlx::query("DELETE FROM tasks WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(_)) => Ok(StatusCode::NO_CONTENT),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Task {} not found", id)})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

fn task_row_to_json(row: &sqlx::postgres::PgRow) -> Value {
    use sqlx::Row;
    json!({
        "id":          row.get::<Uuid, _>(0),
        "project_id":  row.get::<Uuid, _>(1),
        "assignee_id": row.get::<Option<Uuid>, _>(2),
        "title":       row.get::<String, _>(3),
        "description": row.get::<Option<String>, _>(4),
        "status":      row.get::<String, _>(5),
        "priority":    row.get::<String, _>(6),
        "deadline":    row.get::<Option<chrono::DateTime<Utc>>, _>(7),
        "created_at":  row.get::<chrono::DateTime<Utc>, _>(8),
        "updated_at":  row.get::<chrono::DateTime<Utc>, _>(9),
    })
}
