use std::sync::{Arc, RwLock};

use shared_global::eureka::EurekaConfig;

#[derive(Clone)]
pub struct AppState {
    pub eureka_config: Arc<RwLock<EurekaConfig>>,
    // pub jwt_public_key: String,
}

// impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
//     fn jwt_public_key(&self) -> String {
//         self.jwt_public_key.to_string()
//     }
// }
