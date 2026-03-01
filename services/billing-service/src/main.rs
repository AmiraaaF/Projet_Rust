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
    cancel_subscription, check_quota, create_invoice, create_subscription,
    delete_invoice, get_invoice, get_plan, get_subscription, list_invoices,
    list_plans, pay_invoice, update_subscription, AppState,
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
        // ── Health ───────────────────────────────
        .route("/health", get(health_check))
        // ── Plans ────────────────────────────────
        .route("/billing/plans",          get(list_plans))
        .route("/billing/plans/:plan_id", get(get_plan))
        // ── Subscriptions ────────────────────────
        .route("/billing/subscriptions",              post(create_subscription))
        .route("/billing/subscriptions/:user_id",     get(get_subscription))
        .route("/billing/subscriptions/:user_id",     patch(update_subscription))
        .route("/billing/subscriptions/:user_id/cancel", post(cancel_subscription))
        // ── Quotas ───────────────────────────────
        .route("/billing/quota/:user_id", get(check_quota))
        // ── Invoices ─────────────────────────────
        .route("/billing/invoices",               post(create_invoice))
        .route("/billing/invoices/:user_id",      get(list_invoices))
        .route("/billing/invoice/:invoice_id",    get(get_invoice))
        .route("/billing/invoice/:invoice_id/pay", post(pay_invoice))
        .route("/billing/invoice/:invoice_id",    delete(delete_invoice))
        // ── Middleware ───────────────────────────
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new())
                .on_response(tower_http::trace::DefaultOnResponse::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("Failed to bind port 3003");


    println!("   Billing Service  →  http://0.0.0.0:3003 ");
    println!("  GET    /billing/plans                   ");
    println!("  GET    /billing/plans/:plan_id          ");
    println!("  POST   /billing/subscriptions           ");
    println!("  GET    /billing/subscriptions/:user_id  ");
    println!("  PATCH  /billing/subscriptions/:user_id  ");
    println!("  POST   /billing/subscriptions/:id/cancel");
    println!("  GET    /billing/quota/:user_id          ");
    println!("  POST   /billing/invoices                ");
    println!("  GET    /billing/invoices/:user_id       ");
    println!("  GET    /billing/invoice/:invoice_id     ");
    println!("  POST   /billing/invoice/:id/pay         ");
    println!("  DELETE /billing/invoice/:invoice_id     ");
    

    axum::serve(listener, router).await?;
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "billing-service",
        "version": "0.1.0"
    }))
}