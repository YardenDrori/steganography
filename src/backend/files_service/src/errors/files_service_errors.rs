use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum FilesServiceError {
    DatabaseError(sqlx::Error),
    StorageError(String),
    Unauthorized,
    NotFound,
}

impl IntoResponse for FilesServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            Self::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
