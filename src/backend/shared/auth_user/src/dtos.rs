use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Minimal user info shared between auth_service and user_service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBasicInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
}
