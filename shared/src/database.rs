use sqlx::postgres::{PgPool, PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?
        .statement_cache_capacity(256);
    
    PgPool::connect_with(options).await
}

pub async fn init_pool(database_url: &str, pool_size: u32) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(pool_size as u32)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await?;

    Ok(pool)
}
