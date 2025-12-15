use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
    UsernameAlreadyExists,
    DatabaseError(sqlx::Error),
    HashingError(argon2::password_hash::Error),
    InvalidCredentials,
    JwtError(jsonwebtoken::errors::Error),
    ExternalServiceError(String), // For HTTP calls to other microservices
}

impl IntoResponse for UserServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "Email already exists",
            ),
            Self::UsernameAlreadyExists => (
                StatusCode::CONFLICT,
                "Username already exists",
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid credentials",
            ),
            Self::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
            Self::HashingError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
            Self::JwtError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
            Self::ExternalServiceError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to sync with external service",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
