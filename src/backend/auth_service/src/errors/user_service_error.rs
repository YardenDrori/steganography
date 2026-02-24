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
    ParsingError,
    MissingRefreshToken,
}

impl IntoResponse for UserServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            Self::UsernameAlreadyExists => (StatusCode::CONFLICT, "Username already exists"),
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            Self::MissingRefreshToken => (StatusCode::UNAUTHORIZED, "Missing refresh token"),
            Self::DatabaseError(ref e) => {
                tracing::error!(error = ?e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::HashingError(ref e) => {
                tracing::error!(error = ?e, "Password hashing error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::JwtError(ref e) => {
                tracing::error!(error = ?e, "JWT error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::ExternalServiceError(ref msg) => {
                tracing::error!(error = %msg, "External service error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to sync with external service")
            }
            Self::ParsingError => {
                tracing::error!("Parsing error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
