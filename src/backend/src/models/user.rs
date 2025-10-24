#[derive(Debug, Clone)]
pub struct User {
    id: i64,
    user_name: String,
    first_name: String,
    last_name: String,
    is_male: Option<bool>,
    email: String,
    phone_number: Option<String>,
    password_hash: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    is_active: bool,
    is_verified: bool,
}
