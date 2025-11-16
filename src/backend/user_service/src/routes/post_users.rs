use crate::dtos::{UserCreateRequest, UserResponse};
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::service_extractors::InternalService;
use shared_global::errors::ErrorBody;
use shared_global::extractors::ValidatedJson;

pub async fn create_user(
    InternalService: InternalService,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserCreateRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    tracing::info!(
        user_name = %payload.user_name,
        email = %payload.email,
        "Creating user profile"
    );

    let pool = app_state.pool;

    let user_response = user_service::create_user(&pool, &payload)
        .await
        .map_err(|e| match e {
            UserServiceError::EmailAlreadyExists => {
                tracing::warn!(
                    email = %payload.email,
                    "Attempted to create user with existing email"
                );
                (
                    StatusCode::CONFLICT,
                    Json(ErrorBody::new("Email already exists")),
                )
            }
            UserServiceError::UsernameAlreadyExists => {
                tracing::warn!(
                    user_name = %payload.user_name,
                    "Attempted to create user with existing username"
                );
                (
                    StatusCode::CONFLICT,
                    Json(ErrorBody::new("Username already exists")),
                )
            }
            UserServiceError::DatabaseError(error) => {
                tracing::error!(
                    "Database error while creating user {:?}: {:?}",
                    payload.user_name,
                    error
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
            other_error => {
                tracing::error!(
                    "Unexpected error while creating user {:?}: {:?}",
                    payload.user_name,
                    other_error
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal Server Error")),
                )
            }
        })?;

    tracing::info!(
        user_id = %user_response.id,
        user_name = %user_response.user_name,
        "User profile created successfully"
    );

    Ok((StatusCode::CREATED, Json(user_response)))
}
