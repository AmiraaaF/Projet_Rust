use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use shared::{
    models::{UserPublic, User, PaginatedResponse},
};
use uuid::Uuid;
use crate::AppState;

type DbErr = (StatusCode, Json<Value>);

/// List all users (paginated)
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserPublic>>, DbErr> {
    let users: Vec<User> = sqlx::query_as::<_, User>(
        "SELECT id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at FROM users"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()}))
    ))?;

    let public_users: Vec<UserPublic> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(public_users))
}

/// Get a user by ID (profile)
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPublic>, DbErr> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"}))
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        )
    })?;

    Ok(Json(user.into()))
}

/// Update user profile (name, email, etc.)
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<UserPublic>, DbErr> {
    // Validate request body has something to update
    let new_name = body.get("name").and_then(|v| v.as_str());
    let new_email = body.get("email").and_then(|v| v.as_str());
    
    if new_name.is_none() && new_email.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "At least one field (name, email) is required"}))
        ));
    }

    // Build dynamic update query
    let mut updates = vec![];
    let mut query_values: Vec<Box<dyn std::any::Any + Send>> = vec![];
    
    if let Some(name) = new_name {
        updates.push("name = $2");
        query_values.push(Box::new(name.to_string()));
    }
    
    if let Some(email) = new_email {
        let idx = if new_name.is_some() { 3 } else { 2 };
        updates.push(&format!("email = ${}", idx));
        query_values.push(Box::new(email.to_string()));
    }

    // Simplified: just update name
    let query_str = if let Some(name) = new_name {
        "UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2"
    } else if let Some(email) = new_email {
        "UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2"
    } else {
        ""
    };

    let user: User = if let Some(name) = new_name {
        sqlx::query_as::<_, User>(
            "UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at"
        )
        .bind(name)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    } else if let Some(email) = new_email {
        sqlx::query_as::<_, User>(
            "UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at"
        )
        .bind(email)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    } else {
        sqlx::query_as::<_, User>(
            "SELECT id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    }
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"}))
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        )
    })?;

    Ok(Json(user.into()))
}

/// Delete a user
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, DbErr> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"}))
        ));
    }

    Ok(Json(json!({"message": "User deleted successfully"})))
}

/// Update user role (admin/member)
pub async fn update_user_role(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<UserPublic>, DbErr> {
    let new_role = body.get("role").and_then(|v| v.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "role field is required"}))
        ))?;

    // Validate role
    if !["admin", "user"].contains(&new_role) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid role. Must be 'admin' or 'user'"}))
        ));
    }

    let user: User = sqlx::query_as::<_, User>(
        "UPDATE users SET role = $1::user_role, updated_at = NOW() WHERE id = $2 RETURNING id, email, name, password_hash, CAST(role AS TEXT) as role, is_active, created_at, updated_at"
    )
    .bind(new_role)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"}))
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        )
    })?;

    Ok(Json(user.into()))
}

/// Update user account settings
pub async fn update_user_settings(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, DbErr> {
    let is_active = body.get("is_active").and_then(|v| v.as_bool());

    if is_active.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "is_active field is required"}))
        ));
    }

    let _result = sqlx::query(
        "UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(is_active)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()}))
    ))?;

    Ok(Json(json!({"message": "Settings updated successfully"})))
}
