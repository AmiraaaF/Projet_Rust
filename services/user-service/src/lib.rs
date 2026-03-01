use sqlx::PgPool;
use std::sync::Arc;
use shared::auth::AuthService;

pub mod handlers;
pub mod models;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<AuthService>,
}

pub fn init() {
    tracing::info!("User service initialized");
}
