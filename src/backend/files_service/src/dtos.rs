use serde::{Deserialize, Serialize};
use shared_global::dtos::UserResponse;
use validator::Validate;

// DTOs for registration - includes both auth and profile data
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = "validate_username"))]
    pub user_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: String,

    // Profile fields
    #[validate(length(min = 1, max = 50, message = "First name must be between 1 and 50 characters"))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50, message = "Last name must be between 1 and 50 characters"))]
    pub last_name: String,

    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
}

fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    shared_global::validation::validate_username(username)
}

// DTOs for login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub user_name: Option<String>,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,

    pub device_info: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

// DTO for refresh token request
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
    pub refresh_token: String,
}

// DTO for refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// DTO for logout request
#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
    pub refresh_token: String,
}
