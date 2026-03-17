use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use shared::{
    auth::AuthService,
    database::init_pool,
};
use sqlx::PgPool;
use std::sync::Arc;

mod handlers;

use handlers::personal_task;
use personal_task_service::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-key-must-be-32-chars-long-!!".to_string());
    let jwt_expiration: i64 = std::env::var("JWT_EXPIRATION")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);

    let db = init_pool(&database_url, 5)
        .await
        .expect("Failed to initialize database pool");

    let auth = Arc::new(AuthService::new(jwt_secret, jwt_expiration));
    let state = AppState { db, auth };

    let router = Router::new()
        .route("/health", get(health_check))
        .route("/tasks", post(personal_task::create_personal_task))
        .route("/tasks", get(personal_task::list_personal_tasks))
        .route("/tasks/:id", get(personal_task::get_personal_task))
        .route("/tasks/:id", patch(personal_task::update_personal_task))
        .route("/tasks/:id", delete(personal_task::delete_personal_task))
        .route("/tasks/with-deadline", get(personal_task::get_personal_tasks_with_deadline))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3006")
        .await?;
    
    tracing::info!("Task service running on http://0.0.0.0:3006");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "service": "personal-task-service"}))
}
