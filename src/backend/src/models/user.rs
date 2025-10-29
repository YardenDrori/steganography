use chrono::Utc;

#[derive(Debug, Clone)]
pub struct User {
    id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_male: Option<bool>,
    pub email: String,
    pub phone_number: Option<String>,
    password_hash: String, //private as it is
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    is_active: bool,
    is_verified: bool,
}

impl User {
    //getters
    pub fn password_hash(&self) -> String {
        self.password_hash.clone()
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn created_at(&self) -> chrono::DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> chrono::DateTime<Utc> {
        self.updated_at
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }
}
