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
use shared_global::extractors::ValidatedJson;

pub async fn register(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), UserServiceError> {
    let user_service_url: String;
    let jwt_private_key: String;
    {
        let config = app_state.eureka_config.read().unwrap();
        user_service_url = config
            .services
            .get("user_service")
            .ok_or(UserServiceError::ExternalServiceError(
                "user_service not found in eureka".to_string(),
            ))?
            .clone();
        jwt_private_key =
            config
                .jwt_private_key
                .clone()
                .ok_or(UserServiceError::ExternalServiceError(
                    "jwt_private_key not found in eureka".to_string(),
                ))?;
    }
    let pool = &app_state.pool;

    tracing::info!("Registration attempt for username: {}", payload.user_name);

    let user_response = register_user(
        &pool,
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

    // Generate tokens for auto-login after registration
    let access_token =
        token_service::create_access_token(user_response.id, pool, &jwt_private_key).await?;
    let refresh_token = token_service::create_refresh_token(pool, user_response.id, None).await?;

    tracing::info!("User registered successfully with tokens");
    Ok((
        StatusCode::CREATED,
        Json(LoginResponse {
            user: user_response,
            access_token,
            refresh_token,
        }),
    ))
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

    let jwt_private_key: String;
    let user_service_url: String;
    {
        let config = app_state.eureka_config.read().unwrap();
        jwt_private_key =
            config
                .jwt_private_key
                .clone()
                .ok_or(UserServiceError::ExternalServiceError(
                    "jwt_private_key not found in eureka".to_string(),
                ))?;
        user_service_url = config
            .services
            .get("user_service")
            .ok_or(UserServiceError::ExternalServiceError(
                "user_service not found in eureka".to_string(),
            ))?
            .clone();
    }
    let pool = &app_state.pool;

    let login_response = login_user(
        &pool,
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
    let jwt_private_key = app_state
        .eureka_config
        .read()
        .unwrap()
        .jwt_private_key
        .clone()
        .ok_or(UserServiceError::ExternalServiceError(
            "jwt_private_key not found in eureka".to_string(),
        ))?;
    let pool = &app_state.pool;

    let (access_token, refresh_token) =
        token_service::refresh_access_token(pool, &payload.refresh_token, &jwt_private_key).await?;

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
