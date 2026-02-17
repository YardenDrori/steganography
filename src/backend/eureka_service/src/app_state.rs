use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub internal_api_key: String,
    pub registered_services: Arc<RwLock<HashMap<String, String>>>,
}
