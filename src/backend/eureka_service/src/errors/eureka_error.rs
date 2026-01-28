use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::errors::ErrorBody;

#[derive(Debug)]
pub enum EurekaError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    ConfigNotReady,
}

impl IntoResponse for EurekaError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            Self::NotFound(name) => (StatusCode::NOT_FOUND, format!("Service '{}' not found", name)),
            Self::ConfigNotReady => (StatusCode::SERVICE_UNAVAILABLE, "Shared config not yet initialized".to_string()),
        };
        (status, Json(ErrorBody::new(&message))).into_response()
    }
}

impl From<sqlx::Error> for EurekaError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err)
    }
}
