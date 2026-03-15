use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::models::{CreateProjectRequest, Project, UpdateProjectRequest};
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
fn default_limit() -> i64 { 10 }

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

// Helper to extract user_id from JWT token
fn extract_user_id_from_token(headers: &HeaderMap, state: &AppState) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid authorization format"})),
            )
        })?;

    let claims = state.auth.validate_token(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token"})),
        )
    })?;

    Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })
}

pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id_from_token(&headers, &state)?;

    // Check project quota
    let billing_service_url = std::env::var("BILLING_SERVICE_URL")
        .unwrap_or_else(|_| "http://billing-service:3003".to_string());
        
    let quota_resp = state.http_client
        .get(&format!("{}/billing/quota/{}", billing_service_url, user_id))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to contact billing service: {}", e)})),
            )
        })?;
        
    let quota_data: serde_json::Value = quota_resp.json().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to parse billing response"})),
        )
    })?;
    
    let max_projects = quota_data["quotas"]["max_projects"].as_i64().unwrap_or(3);
    
    let current_project_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE owner_id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));
        
    if max_projects != -1 && current_project_count.0 >= max_projects {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": format!("Project limit reached. Upgrade your plan to create more than {} projects.", max_projects)})),
        ));
    }

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (id, owner_id, name, description, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'active'::project_status, NOW(), NOW())
        RETURNING id, owner_id, name, description, status::text AS status, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create project: {}", e)})),
        )
    })?;

    // Send notification
    let _ = state.http_client
        .post("http://notification-service:3004/notifications")
        .json(&json!({
            "user_id": user_id,
            "title": format!("Project created: {}", payload.name),
            "message": format!("You created a new project '{}'", payload.name),
            "notification_type": "in_app"
        }))
        .send()
        .await;

    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Project>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id_from_token(&headers, &state)?;

    let offset = (params.page - 1) * params.limit;

    // Count total projects for this user
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM projects WHERE owner_id = $1"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to count projects: {}", e)})),
        )
    })?;

    // Fetch projects
    let projects = sqlx::query_as::<_, Project>(
        r#"
        SELECT id, owner_id, name, description, status::text AS status, created_at, updated_at
        FROM projects
        WHERE owner_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(params.limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to fetch projects: {}", e)})),
        )
    })?;

    Ok(Json(PaginatedResponse {
        data: projects,
        total: total.0,
        page: params.page,
        limit: params.limit,
    }))
}

pub async fn get_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id_from_token(&headers, &state)?;

    let project = sqlx::query_as::<_, Project>(
        r#"
        SELECT id, owner_id, name, description, status::text AS status, created_at, updated_at
        FROM projects
        WHERE id = $1 AND owner_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Project not found"})),
        )
    })?;

    Ok(Json(project))
}

pub async fn update_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id_from_token(&headers, &state)?;

    let project = sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            status = COALESCE($3::project_status, status),
            updated_at = NOW()
        WHERE id = $4 AND owner_id = $5
        RETURNING id, owner_id, name, description, status::text AS status, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.status)
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Project not found"})),
        )
    })?;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id_from_token(&headers, &state)?;

    let result = sqlx::query(
        "DELETE FROM projects WHERE id = $1 AND owner_id = $2"
    )
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Project not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Project Members handlers

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MemberInfo {
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: Option<String>,
}

pub async fn get_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<MemberInfo>>, (StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let members = sqlx::query_as::<_, MemberInfo>(
        r#"
        SELECT user_id, role, joined_at
        FROM project_members
        WHERE project_id = $1
        "#,
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok(Json(members))
}

pub async fn add_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let role = payload.role.unwrap_or_else(|| "member".to_string());

    sqlx::query(
        r#"
        INSERT INTO project_members (id, project_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (project_id, user_id) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(project_id)
    .bind(payload.user_id)
    .bind(&role)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "Member added successfully"})),
    ))
}

pub async fn remove_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, member_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let _user_id = extract_user_id_from_token(&headers, &state)?;

    let result = sqlx::query(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2"
    )
    .bind(project_id)
    .bind(member_user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Member not found"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
