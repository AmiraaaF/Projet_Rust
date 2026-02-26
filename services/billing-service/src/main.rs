use axum::Router;
use serde_json::json;
use shared::database::init_pool;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db = init_pool(&database_url, 5)
        .await
        .expect("Failed to initialize database pool");

    let state = AppState { db };

    let router = Router::new()
        .route(
            "/health",
            axum::routing::get(|| async {
                axum::Json(json!({
                    "status": "healthy",
                    "service": "billing-service"
                }))
            }),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("Failed to bind port 3003");

    println!("✅ Billing Service starting on http://0.0.0.0:3003");

    axum::serve(listener, router).await?;

    Ok(())
}
