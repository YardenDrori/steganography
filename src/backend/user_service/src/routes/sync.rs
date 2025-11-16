use crate::app_state::AppState;
use crate::dtos::{SyncUserStatusRequest, UserResponse};
use crate::errors::user_service_errors::UserServiceError;
use crate::services::user_service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::service_extractors::InternalService;
use shared_global::errors::ErrorBody;
use shared_global::extractors::ValidatedJson;

pub async fn sync_user_status(
    InternalService: InternalService,
    Path(user_id): Path<i64>,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SyncUserStatusRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    tracing::info!(
        user_id = %user_id,
        is_active = %payload.is_active,
        "Received request to sync user active status"
    );

    let pool = app_state.pool;

    let user_response = user_service::set_user_active_status(&pool, user_id, payload.is_active)
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
                tracing::error!("Database error: {:?}", err);
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

    tracing::info!(
        user_id = %user_id,
        is_active = %payload.is_active,
        "User active status synced successfully"
    );

    Ok((StatusCode::OK, Json(user_response)))
}
