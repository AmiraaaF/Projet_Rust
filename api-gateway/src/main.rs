use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
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
        // Auth routes -> user-service
        .route("/auth/register", post(proxy_to_user_service))
        .route("/auth/login", post(proxy_to_user_service))
        // User routes -> user-service
        .route("/users", get(proxy_to_user_service))
        .route("/users/:id", get(proxy_to_user_service))
        .route("/users/:id", patch(proxy_to_user_service))
        .route("/users/:id", delete(proxy_to_user_service))
        // Project routes -> project-service
        .route("/projects", post(proxy_to_project_service))
        .route("/projects", get(proxy_to_project_service))
        .route("/projects/:id", get(proxy_to_project_service))
        .route("/projects/:id", patch(proxy_to_project_service))
        .route("/projects/:id", delete(proxy_to_project_service))
        .route("/projects/:id/members", get(proxy_to_project_service))
        .route("/projects/:id/members", post(proxy_to_project_service))
        .route("/projects/:id/members/:user_id", delete(proxy_to_project_service))
        .route("/projects/:id/tasks", post(proxy_to_project_service))
        .route("/projects/:id/tasks", get(proxy_to_project_service))
        // Task routes -> project-service
        .route("/tasks/:id", get(proxy_to_project_service))
        .route("/tasks/:id", patch(proxy_to_project_service))
        .route("/tasks/:id", delete(proxy_to_project_service))
        // Billing routes -> billing-service
        .route("/billing/subscriptions/:user_id", get(proxy_to_billing_service))
        .route("/billing/subscriptions/:user_id", post(proxy_to_billing_service))
        .route("/billing/subscriptions/:user_id", delete(proxy_to_billing_service))
        .route("/billing/invoices/:user_id", get(proxy_to_billing_service))
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

// Proxy handlers
async fn proxy_to_user_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state.user_service_url, headers, req).await
}

async fn proxy_to_project_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state.project_service_url, headers, req).await
}

async fn proxy_to_billing_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state.billing_service_url, headers, req).await
}

async fn proxy_request(
    service_url: &str,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let client = reqwest::Client::new();
    let uri = req.uri();
    let method = req.method().clone();
    let path_and_query = uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(uri.path());
    
    let url = format!("{}{}", service_url, path_and_query);
    
    // Extract body
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Build the request
    let mut proxy_req = client.request(method.clone(), &url);
    
    // Forward relevant headers
    if let Some(auth) = headers.get("authorization") {
        proxy_req = proxy_req.header("authorization", auth);
    }
    if let Some(content_type) = headers.get("content-type") {
        proxy_req = proxy_req.header("content-type", content_type);
    }
    
    // Add body if not empty
    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes);
    }
    
    // Send request
    let resp = proxy_req
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to proxy request to {}: {}", url, e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Build response
    let status = resp.status();
    let resp_headers = resp.headers().clone();
    let body_bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    let mut response = Response::builder()
        .status(status);
    
    // Forward content-type header
    if let Some(content_type) = resp_headers.get("content-type") {
        response = response.header("content-type", content_type);
    }
    
    response
        .body(Body::from(body_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
