use axum::{
    routing::{get, post, patch, delete},
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
mod middleware;

use handlers::{auth, user};

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
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/users", get(user::list_users))
        .route("/users/:id", get(user::get_user))
        .route("/users/:id", patch(user::update_user))
        .route("/users/:id", delete(user::delete_user))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new())
                .on_response(tower_http::trace::DefaultOnResponse::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind port 3001");

    println!("✅ User Service starting on http://0.0.0.0:3001");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "user-service",
        "version": "0.1.0"
    }))
}
