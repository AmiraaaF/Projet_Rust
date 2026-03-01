use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use shared::{
    models::{RegisterRequest, LoginRequest, AuthResponse, UserPublic},
    auth::{hash_password, verify_password},
};
use uuid::Uuid;
use crate::AppState;

type DbErr = (StatusCode, Json<serde_json::Value>);

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), DbErr> {
    // Check if user already exists
    let existing: bool = {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(&body.email)
        .fetch_one(&state.db)
        .await
        .map_err(|e: sqlx::Error| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()}))
    ))?
    };

    if existing {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "Email already exists"}))
        ));
    }

    // Hash password
    let password_hash = hash_password(&body.password)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'user', true, $5, $6)
        "#
    )
    .bind(user_id)
    .bind(&body.email)
    .bind(&body.name)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()}))
    ))?
    ;

    // Create default subscription (Free plan)
    sqlx::query(
        r#"
        INSERT INTO subscriptions (user_id, plan, status, auto_renew, max_projects, max_tasks)
        VALUES ($1, 'free', 'active', true, 3, 100)
        "#
    )
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()}))
    ))?
    ;

    // Generate token
    let token: String = state.auth.generate_token(user_id, body.email.clone(), "user".to_string())
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?;

    let user = UserPublic {
        id: user_id,
        email: body.email,
        name: body.name,
        role: "user".to_string(),
        created_at: now,
    };

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user,
        })
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, DbErr> {
    // Find user by email
    let user: Option<(Uuid, String, String, String)> = {
        sqlx::query_as::<_, (Uuid, String, String, String)>(
            "SELECT id, email, name, password_hash FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(&body.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e: sqlx::Error| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?
    };

    let (user_id, email, name, password_hash) = user.ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "Invalid email or password"}))
    ))?;

    let user_id = user_id;

    // Verify password
    let pwd_valid = verify_password(&body.password, &password_hash)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?;

    if !pwd_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"}))
        ));
    }

    // Generate token
    let token: String = state.auth.generate_token(user_id, email.clone(), "user".to_string())
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ))?;

    let user = UserPublic {
        id: user_id,
        email,
        name,
        role: "user".to_string(),
        created_at: chrono::Utc::now(),
    };

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user,
    }))
}
