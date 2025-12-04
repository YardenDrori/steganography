use crate::app_state::AppState;
use crate::dtos::{
    LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest,
};
use crate::errors::user_service_error::UserServiceError;
use crate::services::token_service;
use crate::services::user_service::{login_user, register_user};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use shared_global::dtos::UserResponse;
use shared_global::extractors::ValidatedJson;

pub async fn register(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), UserServiceError> {
    let internal_api_key = app_state.internal_api_key;
    let user_service_url = app_state.user_service_url;
    let pool = &app_state.pool;

    tracing::info!("Registration attempt for username: {}", payload.user_name);

    let user_response = register_user(
        &pool,
        &internal_api_key,
        &user_service_url,
        &payload.user_name,
        &payload.first_name,
        &payload.last_name,
        &payload.email,
        payload.phone_number.as_deref(),
        payload.is_male,
        &payload.password,
    )
    .await?;

    tracing::info!("User registered successfully");
    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), UserServiceError> {
    tracing::info!(
        "Login attempt for email/username: {:?}/{:?}",
        payload.email,
        payload.user_name
    );

    let jwt_private_key = &app_state.jwt_private_key;
    let pool = &app_state.pool;
    let internal_api_key = &app_state.internal_api_key;
    let user_service_url = &app_state.user_service_url;

    let login_response = login_user(
        &pool,
        &internal_api_key,
        &user_service_url,
        payload.email.as_deref(),
        payload.user_name.as_deref(),
        &payload.password,
        payload.device_info.as_deref(),
        &jwt_private_key,
    )
    .await?;

    tracing::info!("User logged in successfully");
    Ok((StatusCode::OK, Json(login_response)))
}

pub async fn refresh(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<RefreshTokenResponse>), UserServiceError> {
    tracing::info!("Token refresh request received");
    let jwt_private_key = &app_state.jwt_private_key;
    let pool = &app_state.pool;

    let (access_token, refresh_token) =
        token_service::refresh_access_token(pool, &payload.refresh_token, jwt_private_key).await?;

    tracing::info!("Token refreshed successfully");
    Ok((
        StatusCode::OK,
        Json(RefreshTokenResponse {
            access_token,
            refresh_token,
        }),
    ))
}

pub async fn logout(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LogoutRequest>,
) -> Result<StatusCode, UserServiceError> {
    tracing::info!("Logout request received");
    let pool = &app_state.pool;

    token_service::revoke_refresh_token(pool, &payload.refresh_token).await?;

    tracing::info!("User logged out successfully");
    Ok(StatusCode::NO_CONTENT)
}
