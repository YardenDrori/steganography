use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_private_key: String,
    pub internal_api_key: String,
    pub user_service_url: String,
}

impl shared_global::auth::internal::HasInternalApiKey for AppState {
    fn internal_api_key(&self) -> String {
        self.internal_api_key.to_string()
    }
}
