use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Creates a PostgreSQL connection pool with standard configuration
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Some(Duration::from_secs(600)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .connect(database_url)
        .await
}
