use crate::app_state::AppState;
use crate::errors::user_service_error::UserServiceError;
use crate::services::token_service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use shared_global::auth::hybrid_extractors::AdminOrInternal;

pub async fn revoke_user_tokens(
    AdminOrInternal(_auth_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Revoking all tokens for user"
    );

    let pool = &app_state.pool;

    token_service::revoke_all_user_tokens(pool, user_id).await?;

    tracing::info!(
        user_id = %user_id,
        "Successfully revoked all tokens for user"
    );

    Ok(StatusCode::NO_CONTENT)
}
