use reqwest;
use shared_global::eureka::EurekaConfig;

#[derive(Clone)]
pub struct AppState {
    pub eureka_configs: EurekaConfig,
    pub client: reqwest::Client,
}
