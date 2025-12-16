use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum FileServiceError {
    DatabaseError(sqlx::Error),
    ExternalServiceError(String), // For HTTP calls to other microservices
}

impl IntoResponse for FileServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::ExternalServiceError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to sync with external service",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
