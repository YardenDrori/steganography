use crate::dtos::{UserCreateRequest, UserResponse};
use crate::services::user_service;
use crate::{app_state::AppState, errors::user_service_errors::UserServiceError};
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::service_extractors::InternalService;
use shared_global::extractors::ValidatedJson;

pub async fn create_user(
    InternalService: InternalService,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserCreateRequest>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    tracing::info!(
        user_name = %payload.user_name,
        email = %payload.email,
        "Creating user profile"
    );

    let pool = app_state.pool;

    let user_response = user_service::create_user(&pool, &payload).await?;

    tracing::info!(
        user_id = %user_response.id,
        user_name = %user_response.user_name,
        "User profile created successfully"
    );

    Ok((StatusCode::CREATED, Json(user_response)))
}
