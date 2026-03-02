use shared_global::eureka::EurekaConfig;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub eureka_config: Arc<RwLock<EurekaConfig>>,
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
