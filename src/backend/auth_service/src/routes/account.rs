use crate::app_state::AppState;
use crate::errors::user_service_error::UserServiceError;
use crate::services::user_service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use shared_global::auth::user_extractors::{AuthenticatedUser, RequireAdmin};

pub async fn deactivate_my_account(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<StatusCode, UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "User deactivating their own account"
    );

    let pool = &app_state.pool;
    let internal_api_key = &app_state.internal_api_key;
    let user_service_url = &app_state.user_service_url;

    user_service::deactivate_user(pool, internal_api_key, user_service_url, user_id).await?;

    tracing::info!(
        user_id = %user_id,
        "User account deactivated successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub async fn activate_user_admin(
    RequireAdmin(_admin_id): RequireAdmin,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Admin activating user account"
    );

    let pool = &app_state.pool;
    let internal_api_key = &app_state.internal_api_key;
    let user_service_url = &app_state.user_service_url;

    user_service::activate_user(pool, internal_api_key, user_service_url, user_id).await?;

    tracing::info!(
        user_id = %user_id,
        "User account activated successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub async fn deactivate_user_admin(
    RequireAdmin(_admin_id): RequireAdmin,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Admin deactivating user account"
    );

    let pool = &app_state.pool;
    let internal_api_key = &app_state.internal_api_key;
    let user_service_url = &app_state.user_service_url;

    user_service::deactivate_user(pool, internal_api_key, user_service_url, user_id).await?;

    tracing::info!(
        user_id = %user_id,
        "User account deactivated successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}
