use std::sync::{Arc, RwLock};

use shared_global::eureka::EurekaConfig;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub eureka_config: Arc<RwLock<EurekaConfig>>,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.jwt_public_key.to_string()
    }
}
