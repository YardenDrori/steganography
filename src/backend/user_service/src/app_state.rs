use sqlx::{Pool, Postgres};

/// Application state shared across all route handlers
/// Contains the database connection pool
#[derive(Clone)]
pub struct AppState {
    pub jwt_public_key: String,
    pub auth_service_url: String,
    pub pool: Pool<Postgres>,
}

// LEGACY READ shared/global/db/pool_provider for info
// impl shared_global::db::pool_provider::HasPgPool for AppState {
//     fn pool(&self) -> &PgPool {
//         &self.pool
//     }
// }

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.jwt_public_key.to_string()
    }
}
