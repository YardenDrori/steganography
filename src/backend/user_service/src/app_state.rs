use sqlx::{PgPool, Pool, Postgres};

/// Application state shared across all route handlers
/// Contains the database connection pool
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl shared_global::db::pool_provider::HasPgPool for AppState {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
