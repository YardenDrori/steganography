use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub(crate) struct UserDbModel {
    pub id: i64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_male: Option<bool>,
    pub email: String,
    pub phone_number: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_verified: bool,
}

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
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
    is_verified: bool,
}

impl User {
    //getters
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn user_name(&self) -> &str {
        &self.user_name
    }
    pub fn first_name(&self) -> &str {
        &self.first_name
    }
    pub fn last_name(&self) -> &str {
        &self.last_name
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn phone_number(&self) -> Option<&str> {
        self.phone_number.as_deref()
    }
    pub fn is_male(&self) -> Option<bool> {
        self.is_male
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Verifies a password against the stored hash
    pub fn verify_password(&self, password: &str) -> Result<bool, argon2::password_hash::Error> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Marks the user's email as verified
    pub fn verify_email(&mut self) {
        self.is_verified = true;
    }

    /// Deactivates the user account
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Reactivates the user account
    pub fn activate(&mut self) {
        self.is_active = true;
    }
}
//auto converts from database struct to service layer struct
impl From<UserDbModel> for User {
    fn from(db: UserDbModel) -> Self {
        User {
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
