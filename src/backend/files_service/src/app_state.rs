use s3::Bucket;
use shared_global::eureka::EurekaConfig;
use sqlx::{Pool, Postgres};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub eureka_config: Arc<RwLock<EurekaConfig>>,
    pub bucket: Bucket,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.eureka_config
            .read()
            .unwrap()
            .jwt_public_key
            .to_string()
    }
}
