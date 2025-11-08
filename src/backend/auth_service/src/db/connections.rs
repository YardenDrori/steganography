use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20) // Max 20 concurrent connections
        .acquire_timeout(Duration::from_secs(5)) // Wait 5s for connection
        .idle_timeout(Duration::from_secs(600)) // Close idle after 10min
        .max_lifetime(Duration::from_secs(1800)) // Recycle after 30min
        .connect(database_url)
        .await
}
