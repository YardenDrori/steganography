use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: i64,
    pub user_name: String,
    pub email: String,
    pub is_active: bool,
    pub is_verified: bool,
}
