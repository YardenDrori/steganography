use crate::models::user::User;
use serde::{Deserialize, Serialize};
pub use shared_global::dtos::UserResponse;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct UserCreateRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = "validate_username"))]
    pub user_name: String,

    #[validate(length(min = 1, max = 50, message = "First name must be between 1 and 50 characters"))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50, message = "Last name must be between 1 and 50 characters"))]
    pub last_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
}

fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    shared_global::validation::validate_username(username)
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 50, message = "First name must be between 1 and 50 characters"))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 50, message = "Last name must be between 1 and 50 characters"))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct SyncUserStatusRequest {
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        UserResponse {
            id: value.id(),
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
