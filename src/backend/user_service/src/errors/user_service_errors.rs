use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum UserServiceError {
    DatabaseError(sqlx::Error),
    Unauthorized,
    NotFound,
    EmailAlreadyExists,
    UsernameAlreadyExists,
}

impl IntoResponse for UserServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "Resource not found",
            ),
            Self::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "Email already exists",
            ),
            Self::UsernameAlreadyExists => (
                StatusCode::CONFLICT,
                "Username already exists",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
