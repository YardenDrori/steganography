use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum StegServiceError {
    ExternalServiceError(String),
}

impl IntoResponse for StegServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::ExternalServiceError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to sync with external service",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
