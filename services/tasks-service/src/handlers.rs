use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::models::{CreateTaskRequest, TaskFilters};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String, 
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

pub async fn list_tasks(State(state): State<AppState>, Query(filters): Query<TaskFilters>,headers: axum::http::HeaderMap,) -> Result<Json<Value>, (StatusCode, Json<Value>)> {

    //Extraire l'utilisateur connecté
    let token = match headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token manquant"})),
        )),
    };

    let auth_service = shared::auth::AuthService::new(state.jwt_secret.clone(), 3600);
    let claims = match auth_service.validate_token(&token) {
        Ok(c) => c,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide ou expiré"})),
        )),
    };

    let connected_user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide"})),
        )),
    };

    // 2. Récupérer les tâches des projets où l'utilisateur connecté est membre.
    //    soit il est propriétaire soit il est membre 
    match sqlx::query(
        r#"
        SELECT t.id, t.project_id, t.assignee_id, t.title, t.description,
               CAST(t.status AS TEXT), CAST(t.priority AS TEXT),
               t.deadline, t.created_at, t.updated_at,
               u.name AS assignee_name, p.name AS project_name
        FROM tasks t
        LEFT JOIN users u ON t.assignee_id = u.id
        LEFT JOIN projects p ON t.project_id = p.id
        WHERE t.project_id IN (
            SELECT id FROM projects WHERE owner_id = $1
            UNION
            SELECT project_id FROM project_members WHERE user_id = $1
        )
          AND ($2::uuid IS NULL OR t.assignee_id = $2)
          AND ($3::text IS NULL OR CAST(t.status AS TEXT) = $3)
          AND ($4::uuid IS NULL OR t.project_id = $4)
        ORDER BY t.created_at DESC
        "#,
    )
    .bind(connected_user_id)
    .bind(filters.assignee_id)
    .bind(filters.status.as_deref())
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
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {


    let token = match headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token manquant"})),
        )),
    };

    let auth_service = shared::auth::AuthService::new(state.jwt_secret.clone(), 3600);
    let claims = match auth_service.validate_token(&token) {
        Ok(c) => c,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide ou expiré"})),
        )),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(uid) => uid,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide"})),
        )),
    };

    let task_row = match sqlx::query("SELECT assignee_id FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Task {} not found", id)})),
        )),
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    };

    let assignee_id: Option<Uuid> = task_row.get(0);
    if assignee_id != Some(user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Tu n'es pas autorisé à modifier cette tâche"})),
        ));
    }
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
//seul le proprietaire du projet peut supprimie les taches de ce projet si pas proprio il peut pas 
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {

    let token = match headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token manquant"})),
        )),
    };
    let auth_service = shared::auth::AuthService::new(state.jwt_secret.clone(), 3600);
    let claims = match auth_service.validate_token(&token) {
        Ok(c) => c,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide ou expiré"})),
        )),
    };
    let connected_user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Token invalide"})),
        )),
    };

    let row = sqlx::query("SELECT project_id FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    let project_id: Uuid = match row {
        Some(row) => row.get(0),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Task {} not found", id)})),
            ))
        }
    };
    
    let owner_row = sqlx::query("SELECT owner_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    let owner_id: Uuid = match owner_row {
        Some(row) => row.get(0),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Project {} not found", project_id)})),
            ))
        }
    };
    if connected_user_id != owner_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Seul le propriétaire du projet peut supprimer cette tâche."})),
        ));
    }

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
        "assignee_name": row.try_get::<Option<String>, _>(10).unwrap_or(None),
        "project_name": row.try_get::<Option<String>, _>(11).unwrap_or(None),
    })
}
