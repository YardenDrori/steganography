pub enum UserServiceError {
    EmailAlreadyExists,
    UsernameAlreadyExists,
    DatabaseError(sqlx::Error),
    HashingError(bcrypt::BcryptError),
}

