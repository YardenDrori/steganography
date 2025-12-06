use crate::entities::user::UserEntity;
use chrono::{DateTime, Utc};

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

#[allow(dead_code)]
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
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    /// Marks the user's email as verified
    pub fn verify_email(&mut self) {
        self.is_verified = true;
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, argon2::password_hash::Error> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
//auto converts from database entity to domain model
impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        User {
            id: entity.id,
            user_name: entity.user_name,
            first_name: entity.first_name,
            last_name: entity.last_name,
            is_male: entity.is_male,
            email: entity.email,
            phone_number: entity.phone_number,
            password_hash: entity.password_hash,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            is_active: entity.is_active,
            is_verified: entity.is_verified,
        }
    }
}
