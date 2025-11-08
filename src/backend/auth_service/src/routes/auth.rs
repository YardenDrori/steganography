use crate::app_state::AppState;
use crate::dtos::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest};
use crate::errors::user_service_error::UserServiceError;
use crate::services::token_service;
use crate::services::user_service::{login_user, register_user};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use shared::dtos::UserResponse;
use shared::errors::ErrorBody;
use validator::Validate;

pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    tracing::info!("Registration attempt for username: {}", payload.user_name);
    // Validate input at handler layer (input validation, not business logic)
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody::new(&format!("{:?}", e))),
        )
    })?;

    let pool = &app_state.pool;

    let user_response = register_user(&pool, payload).await.map_err(|e| match e {
        UserServiceError::EmailAlreadyExists => {
            tracing::warn!("Registration attempt with existing email");
            (
                StatusCode::CONFLICT,
                Json(ErrorBody::new("Email or username already exists")),
            )
        }
        UserServiceError::UsernameAlreadyExists => {
            tracing::warn!("Registration attempt with existing username");
            (
                StatusCode::CONFLICT,
                Json(ErrorBody::new("Email or username already exists")),
            )
        }
        UserServiceError::DatabaseError(err) => {
            tracing::error!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
        UserServiceError::HashingError(err) => {
            tracing::error!("Hashing error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
        other_error => {
            tracing::error!("Unexpected error: {:?}", other_error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
    })?;

    tracing::info!("User registered successfully");
    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<ErrorBody>)> {
    tracing::info!("Login attempt for email/username: {:?}/{:?}", payload.email, payload.user_name);
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody::new(&format!("{:?}", e))),
        )
    })?;

    let jwt_secret = &app_state.jwt_secret;
    let pool = &app_state.pool;

    let login_response = login_user(&pool, payload, &jwt_secret)
        .await
        .map_err(|e| match e {
            UserServiceError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Invalid credentials")),
            ),
            UserServiceError::HashingError(err) => {
                tracing::error!("Hashing error {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
            UserServiceError::JwtError(err) => {
                tracing::error!("user service error {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
            other_error => {
                tracing::error!("Unexpected error {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
        })?;

    tracing::info!("User logged in successfully");
    Ok((StatusCode::OK, Json(login_response)))
}

pub async fn refresh(
    State(app_state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<RefreshTokenResponse>), (StatusCode, Json<ErrorBody>)> {
    tracing::info!("Token refresh request received");
    let jwt_secret = &app_state.jwt_secret;
    let pool = &app_state.pool;

    let (access_token, refresh_token) = token_service::refresh_access_token(
        pool,
        &payload.refresh_token,
        jwt_secret,
    )
    .await
    .map_err(|e| match e {
        UserServiceError::InvalidCredentials => {
            tracing::warn!("Invalid or expired refresh token used");
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Invalid or expired refresh token")),
            )
        }
        UserServiceError::DatabaseError(err) => {
            tracing::error!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
        UserServiceError::JwtError(err) => {
            tracing::error!("JWT error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
        other_error => {
            tracing::error!("Unexpected error: {:?}", other_error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Internal server error")),
            )
        }
    })?;

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
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    tracing::info!("Logout request received");
    let pool = &app_state.pool;

    token_service::revoke_refresh_token(pool, &payload.refresh_token)
        .await
        .map_err(|e| match e {
            UserServiceError::InvalidCredentials => {
                tracing::warn!("Logout attempt with invalid refresh token");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorBody::new("Invalid refresh token")),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
            other_error => {
                tracing::error!("Unexpected error: {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Internal server error")),
                )
            }
        })?;

    tracing::info!("User logged out successfully");
    Ok(StatusCode::NO_CONTENT)
}
