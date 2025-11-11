use serde::{Deserialize, Serialize};
use shared_global::dtos::UserResponse;
use validator::Validate;

// DTOs for registration - auth_service only handles authentication data
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
}

fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    // Only allow alphanumeric, underscores, and hyphens
    if username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(
            validator::ValidationError::new("username_invalid").with_message(
                std::borrow::Cow::Borrowed(
                    "Username can only contain letters, numbers, underscores, and hyphens",
                ),
            ),
        )
    }
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
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// DTO for refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// DTO for logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}
