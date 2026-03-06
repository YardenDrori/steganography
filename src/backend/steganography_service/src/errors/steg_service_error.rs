use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum StegServiceError {
    InvalidPayload,
    ExternalServiceError(String),
    ParsingError,
    FileError,
    EurekaConfigError,
    FfmpegError(ffmpeg_next::Error),
    Other(String),
    UnsupportedCodec,
}

impl IntoResponse for StegServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidPayload => (StatusCode::BAD_REQUEST, "Invalid payload"),
            Self::UnsupportedCodec => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::FfmpegError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::EurekaConfigError => (StatusCode::BAD_GATEWAY, "Internal server error"),
            Self::ParsingError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::FileError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::ExternalServiceError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to sync with external service",
            ),
            Self::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to sync with external service",
            ),
        };

        (status, Json(ErrorBody::new(message))).into_response()
    }
}
