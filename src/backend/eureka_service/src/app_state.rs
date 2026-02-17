use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::time::Instant;

#[derive(Clone)]
pub struct ServiceEntry {
    pub service_url: String,
    pub last_heartbeat: Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub registered_services: Arc<RwLock<HashMap<String, ServiceEntry>>>,
}
