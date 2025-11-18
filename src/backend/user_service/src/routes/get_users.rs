use crate::dtos::UserResponse;
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::hybrid_extractors::AdminOrInternal;
use shared_global::auth::user_extractors::AuthenticatedUser;

pub async fn get_current_profile(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    let pool = app_state.pool;

    let user_response = user_service::get_user(&pool, user_id).await?;

    Ok((StatusCode::OK, Json(user_response)))
}

pub async fn get_user(
    AdminOrInternal(_maybe_admin_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    let pool = app_state.pool;

    let user_response = user_service::get_user(&pool, user_id).await?;

    Ok((StatusCode::OK, Json(user_response)))
}
