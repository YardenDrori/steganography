use crate::app_state::AppState;
use crate::dtos::{LoginRequest, LoginResponse, RefreshTokenResponse, RegisterRequest};
use crate::errors::user_service_error::UserServiceError;
use crate::services::token_service;
use crate::services::user_service::{login_user, register_user};
use axum::extract::State;
use axum::http::header::{HeaderMap, SET_COOKIE};
use axum::http::StatusCode;
use axum::Json;
use shared_global::extractors::ValidatedJson;

fn build_refresh_cookie(token: &str) -> String {
    format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        token,
        7 * 24 * 60 * 60
    )
}

fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())?
        .split(';')
        .find(|part| part.trim().starts_with("refresh_token="))
        .map(|part| part.trim().trim_start_matches("refresh_token=").to_string())
}

pub async fn register(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResponse>), UserServiceError> {
    let user_service_url: String;
    let jwt_private_key: String;
    {
        tracing::info!("received register request with values of:\n{:?}", payload);
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

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&refresh_token).parse().map_err(|_| {
            tracing::error!("failed to parse refresh token to axum object");
            UserServiceError::ParsingError
        })?,
    );

    tracing::info!("User registered successfully with tokens");
    Ok((
        StatusCode::CREATED,
        headers,
        Json(LoginResponse {
            user: user_response,
            access_token,
        }),
    ))
}

pub async fn login(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResponse>), UserServiceError> {
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

    let (login_response, refresh_token) = login_user(
        &pool,
        &user_service_url,
        payload.email.as_deref(),
        payload.user_name.as_deref(),
        &payload.password,
        payload.device_info.as_deref(),
        &jwt_private_key,
    )
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&refresh_token).parse().map_err(|_| {
            tracing::error!("failed to parse refresh token to axum object");
            UserServiceError::ParsingError
        })?,
    );

    tracing::info!("User logged in successfully");
    Ok((StatusCode::OK, headers, Json(login_response)))
}

pub async fn refresh(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, HeaderMap, Json<RefreshTokenResponse>), UserServiceError> {
    tracing::info!("Token refresh request received");

    let refresh_token =
        extract_refresh_token(&headers).ok_or(UserServiceError::MissingRefreshToken)?;

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

    let (access_token, new_refresh_token) =
        token_service::refresh_access_token(pool, &refresh_token, &jwt_private_key).await?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&new_refresh_token)
            .parse()
            .map_err(|_| UserServiceError::ParsingError)?,
    );

    tracing::info!("Token refreshed successfully");
    Ok((
        StatusCode::OK,
        response_headers,
        Json(RefreshTokenResponse { access_token }),
    ))
}

pub async fn logout(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, HeaderMap), UserServiceError> {
    tracing::info!("Logout request received");

    let refresh_token =
        extract_refresh_token(&headers).ok_or(UserServiceError::MissingRefreshToken)?;

    let pool = &app_state.pool;
    token_service::revoke_refresh_token(pool, &refresh_token).await?;

    // Clear the cookie by setting Max-Age=0
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        SET_COOKIE,
        "refresh_token=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0"
            .parse()
            .map_err(|_| UserServiceError::ParsingError)?,
    );

    tracing::info!("User logged out successfully");
    Ok((StatusCode::NO_CONTENT, response_headers))
}
