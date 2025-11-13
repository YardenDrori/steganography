use crate::dtos::{UserCreateRequest, UserResponse};
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::service_extractors::InternalService;
use shared_global::errors::ErrorBody;

pub async fn create_user(
    InternalService: InternalService,
    State(app_state): State<AppState>,
    Json(payload): Json<UserCreateRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let pool = app_state.pool;

    let user_response = user_service::create_user(&pool, &payload)
        .await
        .map_err(|e| match e {
            UserServiceError::DatabaseError(error) => {
                tracing::error!(
                    "Error {:?} while creating user {:?}",
                    error,
                    payload.user_name
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!(
                    "Unexpected Error {:?} while creating user {:?}",
                    other_error,
                    payload.user_name
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    Ok((StatusCode::CREATED, Json(user_response)))
}
