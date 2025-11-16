use crate::app_state::AppState;
use crate::errors::user_service_error::UserServiceError;
use crate::services::token_service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::hybrid_extractors::AdminOrInternal;
use shared_global::errors::ErrorBody;

pub async fn revoke_user_tokens(
    AdminOrInternal(_auth_id): AdminOrInternal,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    tracing::info!(
        user_id = %user_id,
        "Revoking all tokens for user"
    );

    let pool = &app_state.pool;

    token_service::revoke_all_user_tokens(pool, user_id)
        .await
        .map_err(|e| match e {
            UserServiceError::DatabaseError(err) => {
                tracing::error!("Database error while revoking tokens: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!("Unexpected error while revoking tokens: {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    tracing::info!(
        user_id = %user_id,
        "Successfully revoked all tokens for user"
    );

    Ok(StatusCode::NO_CONTENT)
}
