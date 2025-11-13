use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_secret: String,
    pub internal_api_key: String,
    pub user_service_url: String,
}
