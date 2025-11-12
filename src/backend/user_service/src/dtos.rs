use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        UserResponse {
            user_name: value.user_name().to_string(),
            first_name: value.first_name().to_string(),
            last_name: value.last_name().to_string(),
            email: value.email().to_string(),
            phone_number: value.phone_number().map(|s| s.to_string()),
            is_male: value.is_male(),
            is_verified: value.is_verified(),
            created_at: value.created_at(),
            updated_at: value.updated_at(),
        }
    }
}
