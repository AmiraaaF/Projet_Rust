pub mod handlers;
pub mod models;

use sqlx::PgPool;
use std::sync::Arc;
use shared::auth::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<AuthService>,
}

pub fn init() {
    tracing::info!("Task service initialized");
}
