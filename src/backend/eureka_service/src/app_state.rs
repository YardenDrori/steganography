use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub internal_api_key: String,
    pub service_host: String,
}
