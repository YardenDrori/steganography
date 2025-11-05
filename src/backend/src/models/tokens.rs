use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub(crate) struct refresh_token_DBModel {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub device_info: String
}

#[derive(Debug, Clone)]
pub struct refresh_token {
    id: i64,
    user_name: String,
    first_name: String,
    last_name: String,
    is_male: Option<bool>,
    email: String,
    phone_number: Option<String>,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
    is_verified: bool,
}

#[allow(dead_code)]
impl refresh_token {
    todo!()
}
//auto converts from database struct to service layer struct
impl From<refresh_token_DBModel> for refresh_token {
    fn from(db: refresh_token_DBModel) -> Self {
        refresh_token {
            id: db.id,
            user_name: db.user_name,
            first_name: db.first_name,
            last_name: db.last_name,
            is_male: db.is_male,
            email: db.email,
            phone_number: db.phone_number,
            password_hash: db.password_hash,
            created_at: db.created_at,
            updated_at: db.updated_at,
            is_active: db.is_active,
            is_verified: db.is_verified,
        }
    }
}
