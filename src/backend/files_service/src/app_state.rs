use minior::Minio;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_public_key: String,
    pub minio: Arc<Minio>,
    pub minio_bucket: String,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.jwt_public_key.to_string()
    }
}
