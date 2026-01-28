use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum FileServiceError {
    DatabaseError(sqlx::Error),
    NotFound,
    Unauthorized,
    MinioError(String),
    FileNotReady,
    FileAlreadyConfirmed,
    ExternalServiceError(String),
}

impl IntoResponse for FileServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::NotFound => (StatusCode::NOT_FOUND, "File not found"),
            Self::Unauthorized => (StatusCode::FORBIDDEN, "You do not have access to this file"),
            Self::MinioError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage service error"),
            Self::FileNotReady => (StatusCode::CONFLICT, "File has not been uploaded to storage yet"),
            Self::FileAlreadyConfirmed => (StatusCode::CONFLICT, "File upload has already been confirmed"),
            Self::ExternalServiceError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to communicate with external service",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}

impl From<sqlx::Error> for FileServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err)
    }
}
