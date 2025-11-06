use serde::{Deserialize, Serialize};
use shared::dtos::UserResponse;

// DTOs for registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
}

// DTOs for login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub user_name: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
}
