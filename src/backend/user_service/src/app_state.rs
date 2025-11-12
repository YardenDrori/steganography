use sqlx::{Pool, Postgres};

/// Application state shared across all route handlers
/// Contains the database connection pool
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub pool: Pool<Postgres>,
}

// LEGACY READ shared/global/db/pool_provider for info
// impl shared_global::db::pool_provider::HasPgPool for AppState {
//     fn pool(&self) -> &PgPool {
//         &self.pool
//     }
// }

impl shared_global::auth::jwt::HasJwtSecret for AppState {
    fn jwt_secret(&self) -> String {
        self.jwt_secret.to_string()
    }
}
