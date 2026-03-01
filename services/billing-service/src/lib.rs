pub mod handlers;
pub mod models;

pub use handlers::AppState;

pub fn init() {
    tracing::info!("Billing service initialized");
}
