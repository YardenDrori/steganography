#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
    UsernameAlreadyExists,
    DatabaseError(sqlx::Error),
    HashingError(bcrypt::BcryptError),
    InvalidCredentials,
    JwtError(jsonwebtoken::errors::Error),
}
