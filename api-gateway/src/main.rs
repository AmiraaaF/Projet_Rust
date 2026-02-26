use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use shared::auth::AuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub user_service_url: String,
    pub project_service_url: String,
    pub billing_service_url: String,
    pub notification_service_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .init();

    dotenv::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-key-must-be-32-chars-long-!!".to_string());
    let jwt_expiration: i64 = std::env::var("JWT_EXPIRATION")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);

    let user_service_url = std::env::var("USER_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    let project_service_url = std::env::var("PROJECT_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3002".to_string());
    let billing_service_url = std::env::var("BILLING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3003".to_string());
    let notification_service_url = std::env::var("NOTIFICATION_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3004".to_string());

    let auth = Arc::new(AuthService::new(jwt_secret, jwt_expiration));
    let state = AppState {
        auth,
        user_service_url,
        project_service_url,
        billing_service_url,
        notification_service_url,
    };

    let router = Router::new()
        .route("/health", get(health_check))
        .route("/health/services", get(check_services))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new())
                .on_response(tower_http::trace::DefaultOnResponse::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    println!("✅ API Gateway starting on http://0.0.0.0:3000");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "api-gateway"
        })),
    )
}

async fn check_services(State(state): State<AppState>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let mut results = json!({});

    if let Ok(resp) = client
        .get(&format!("{}/health", state.user_service_url))
        .send()
        .await
    {
        results["user_service"] = json!({"status": resp.status().to_string()});
    } else {
        results["user_service"] = json!({"status": "down"});
    }

    if let Ok(resp) = client
        .get(&format!("{}/health", state.project_service_url))
        .send()
        .await
    {
        results["project_service"] = json!({"status": resp.status().to_string()});
    } else {
        results["project_service"] = json!({"status": "down"});
    }

    (StatusCode::OK, Json(results))
}
