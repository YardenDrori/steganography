#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
    UsernameAlreadyExists,
    DatabaseError(sqlx::Error),
    HashingError(argon2::password_hash::Error),
    InvalidCredentials,
    JwtError(jsonwebtoken::errors::Error),
}
