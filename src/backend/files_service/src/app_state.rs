use shared_global::eureka::EurekaConfig;
use sqlx::{Postgres, Pool};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub eureka_config: Arc<RwLock<EurekaConfig>>,
}
