use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use shared::database::init_pool;
use sqlx::PgPool;

mod handlers;
mod models;

use handlers::{
    create_task, delete_task, get_task, list_tasks, mark_task_done, task_stats, AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db = init_pool(&database_url, 5)
        .await
        .expect("Failed to initialize database pool");

    let state = AppState { db };

    let router = Router::new()
        .route("/health", get(health_check))
        .route("/tasks",       post(create_task))
        .route("/tasks",       get(list_tasks))
        .route("/tasks/stats", get(task_stats))
        .route("/tasks/:id",      get(get_task))
        .route("/tasks/:id/done", patch(mark_task_done))
        .route("/tasks/:id",      delete(delete_task))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new())
                .on_response(tower_http::trace::DefaultOnResponse::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004")
        .await
        .expect("Failed to bind port 3004");

    println!("  Task Service  →  http://0.0.0.0:3004");
    println!("  POST   /tasks");
    println!("  GET    /tasks");
    println!("  GET    /tasks/:id");
    println!("  PATCH  /tasks/:id");
    println!("  DELETE /tasks/:id");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "taks-service",
        "version": "0.1.0"
    }))
}
