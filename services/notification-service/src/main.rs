use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use shared::database::init_pool;
use sqlx::PgPool;

mod handlers;

use handlers::{
    create_notification, delete_all_read, delete_notification, get_notification,
    get_stats, list_notifications, mark_all_read, mark_read, send_event, AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = init_pool(&database_url, 5)
        .await
        .expect("Failed to initialize database pool");

    let state = AppState { db };

    let router = Router::new()
        .route("/health", get(health_check))
        .route("/notifications/:user_id",          get(list_notifications))
        .route("/notifications/:user_id/stats",    get(get_stats))
        .route("/notifications/:user_id/read-all", post(mark_all_read))
        .route("/notifications/:user_id/clear-read", delete(delete_all_read))
        .route("/notification",                    post(create_notification))
        .route("/notification/:id",                get(get_notification))
        .route("/notification/:id/read",           patch(mark_read))
        .route("/notification/:id",                delete(delete_notification))
        .route("/events",                          post(send_event))
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

    println!("   Notification Service  →  http://0.0.0.0:3004");
    println!("  GET    /notifications/:user_id");
    println!("  GET    /notifications/:user_id/stats");
    println!("  POST   /notifications/:user_id/read-all");
    println!("  DELETE /notifications/:user_id/clear-read");
    println!("  POST   /notification");
    println!("  GET    /notification/:id");
    println!("  PATCH  /notification/:id/read");
    println!("  DELETE /notification/:id");
    println!("  POST   /events  (event bus)");

    axum::serve(listener, router).await?;
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status":  "healthy",
        "service": "notification-service",
        "version": "0.1.0",
    }))
}