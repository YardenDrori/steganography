use crate::dtos::UserResponse;
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::user_extractors::RequireAdmin;
use shared_global::{auth::user_extractors::AuthenticatedUser, errors::ErrorBody};

pub async fn get_current_profile(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let pool = app_state.pool;

    let user_response = user_service::get_user(&pool, user_id)
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => {
                tracing::error!("user {} not found", user_id);
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorBody::new("could not find user")),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!("Unexpected error: {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    Ok((StatusCode::OK, Json(user_response)))
}

pub async fn get_user(
    RequireAdmin(_auth_user_id): RequireAdmin,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let pool = app_state.pool;

    let user_response = user_service::get_user(&pool, user_id)
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => {
                tracing::error!("user {} not found", user_id);
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorBody::new("could not find user")),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!("Unexpected error: {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    Ok((StatusCode::OK, Json(user_response)))
}
