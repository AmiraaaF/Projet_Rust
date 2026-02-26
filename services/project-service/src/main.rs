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

use handlers::{project, task};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<AuthService>,
}

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
        .route("/projects", post(project::create_project))
        .route("/projects", get(project::list_projects))
        .route("/projects/:id", get(project::get_project))
        .route("/projects/:id", patch(project::update_project))
        .route("/projects/:id", delete(project::delete_project))
        .route("/projects/:id/members", get(project::get_members))
        .route("/projects/:id/members", post(project::add_member))
        .route("/projects/:id/members/:user_id", delete(project::remove_member))
        .route("/projects/:id/tasks", post(task::create_task))
        .route("/projects/:id/tasks", get(task::list_tasks))
        .route("/tasks/:id", get(task::get_task))
        .route("/tasks/:id", patch(task::update_task))
        .route("/tasks/:id", delete(task::delete_task))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new())
                .on_response(tower_http::trace::DefaultOnResponse::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("Failed to bind port 3002");

    println!("✅ Project Service starting on http://0.0.0.0:3002");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "project-service",
        "version": "0.1.0"
    }))
}
