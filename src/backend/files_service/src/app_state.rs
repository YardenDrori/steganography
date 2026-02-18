use minior::Minio;
use shared_global::eureka::EurekaConfig;
use sqlx::{Pool, Postgres};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub minio: Arc<Minio>,
    pub minio_bucket: String,
    pub eurekea_config: Arc<RwLock<EurekaConfig>>,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.eurekea_config
            .read()
            .unwrap()
            .jwt_public_key
            .to_string()
    }
}
