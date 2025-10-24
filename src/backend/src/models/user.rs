use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_male: Option<bool>,
    pub email: String,
    pub phone_number: Option<String>,
    pub password_hash: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_active: bool,
    pub is_verified: bool,
}

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
#[derive(Debug, Serialize)] //DTO
pub struct UserResponse {
    pub id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_male: Option<bool>,
    pub created_at: chrono::NaiveDateTime,
    pub is_active: bool,
    pub is_verified: bool,
}
