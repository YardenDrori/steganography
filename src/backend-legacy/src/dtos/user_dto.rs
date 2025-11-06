use chrono::Utc;
use serde::{Deserialize, Serialize};

//DTOS - signup
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
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
    pub created_at: chrono::DateTime<Utc>,
    pub is_active: bool,
    pub is_verified: bool,
}

//DTOS - login
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
    pub refresh_token: String,
}
