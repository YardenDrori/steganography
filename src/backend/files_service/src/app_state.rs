use minior::Minio;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_public_key: String,
    pub internal_api_key: String,
    pub user_service_url: String,
    pub auth_service_url: String,
    pub minio: Minio,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.jwt_public_key.to_string()
    }
}

impl shared_global::auth::internal::HasInternalApiKey for AppState {
    fn internal_api_key(&self) -> String {
        self.internal_api_key.to_string()
    }
}
