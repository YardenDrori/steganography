use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use shared_global::auth::hybrid_extractors::AdminOrInternal;
use shared_global::errors::ErrorBody;

pub async fn delete_user(
    AdminOrInternal(_maybe_admin_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let pool = app_state.pool;

    user_service::delete_user(&pool, user_id)
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => {
                tracing::error!("User {} not found", user_id);
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorBody::new("User not found")),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("Database error deleting user {}: {:?}", user_id, err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!(
                    "Unexpected error deleting user {}: {:?}",
                    user_id,
                    other_error
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}
