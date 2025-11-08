use serde::{Deserialize, Serialize};
use shared::dtos::UserResponse;
use validator::Validate;

// DTOs for registration
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = "validate_username"))]
    pub user_name: String,

    #[validate(length(
        min = 1,
        max = 100,
        message = "First name must be between 1 and 100 characters"
    ))]
    pub first_name: String,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Last name must be between 1 and 100 characters"
    ))]
    pub last_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: String,

    #[validate(length(min = 10, max = 10, message = "Phone number must have 10 characters"))]
    #[validate(custom(function = "validate_phone_number"))]
    pub phone_number: Option<String>,

    pub is_male: Option<bool>,
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

fn validate_phone_number(phone_num: &str) -> Result<(), validator::ValidationError> {
    // Only allow numbers
    if phone_num.chars().all(|c| c.is_numeric()) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("username_invalid")
            .with_message(std::borrow::Cow::Borrowed("phone number can only numbers.")))
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
