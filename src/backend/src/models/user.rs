use chrono::Utc;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_male: Option<bool>,
    pub email: String,
    pub phone_number: Option<String>,
    password_hash: String, //private as it is
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_active: bool,
    pub is_verified: bool,
}

impl User {
    pub fn password_hash(&self) -> String {
        self.password_hash.clone()
    }
}
