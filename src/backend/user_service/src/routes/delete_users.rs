use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use shared_global::auth::hybrid_extractors::AdminOrInternal;

pub async fn delete_user(
    AdminOrInternal(_maybe_admin_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, UserServiceError> {
    let pool = app_state.pool;

    user_service::delete_user(&pool, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
