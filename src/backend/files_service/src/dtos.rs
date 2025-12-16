use serde::{Deserialize, Serialize};
use shared_global::dtos::UserResponse;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct PrepareResponse {
    pub url: String,
    pub file_id: i64,
}
