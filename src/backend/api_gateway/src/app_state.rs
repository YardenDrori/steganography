use reqwest;
use shared_global::eureka::EurekaConfig;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub eureka_configs: Arc<RwLock<EurekaConfig>>,
    pub client: reqwest::Client,
}
