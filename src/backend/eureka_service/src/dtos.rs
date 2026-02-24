use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub jwt_public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_private_key: Option<String>,
    pub jwt_duration_access_and_refresh: Option<(i64, i64)>,
    pub services: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Service name must be between 1 and 100 characters"
    ))]
    pub service_name: String,
    #[validate(length(
        min = 1,
        max = 500,
        message = "Service URL must be between 1 and 500 characters"
    ))]
    pub service_url: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub service_url: String,
}

// #[derive(Debug, Serialize)]
// pub struct DiscoverResponse {
//     pub service_name: String,
//     pub service_url: String,
// }
