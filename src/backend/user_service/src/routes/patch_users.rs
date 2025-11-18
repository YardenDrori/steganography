use crate::dtos::{UpdateUserRequest, UserResponse};
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::hybrid_extractors::AdminOrInternal;
use shared_global::auth::user_extractors::AuthenticatedUser;
use shared_global::extractors::ValidatedJson;

/// Update current user's own profile
pub async fn update_my_profile(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "User updating their own profile"
    );

    let pool = app_state.pool;

    let user_response = user_service::update_user(&pool, user_id, &payload).await?;

    Ok((StatusCode::OK, Json(user_response)))
}

/// Update any user's profile (admin or internal service only)
pub async fn update_user(
    AdminOrInternal(_auth_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Admin/internal updating user profile"
    );

    let pool = app_state.pool;

    let user_response = user_service::update_user(&pool, user_id, &payload).await?;

    Ok((StatusCode::OK, Json(user_response)))
}
