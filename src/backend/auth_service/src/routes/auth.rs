use crate::app_state::AppState;
use crate::dtos::{LoginRequest, LoginResponse, RegisterRequest};
use crate::errors::user_service_error::UserServiceError;
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

    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<ErrorBody>)> {
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

    Ok((StatusCode::OK, Json(login_response)))
}
