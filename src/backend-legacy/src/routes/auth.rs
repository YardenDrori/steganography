use crate::AppState;
use crate::dtos::user_dto::{LoginRequest, LoginResponse, RegisterRequest, UserResponse};
use crate::errors::error_body::ErrorBody;
use crate::errors::user_service_error::UserServiceError;
use crate::services::user_service::{login_user, register_user};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let pool = &app_state.pool;

    let user_response = register_user(&pool, payload).await.map_err(|e| match e {
        //cannot prevent enumeration without email verification so for now we dont try to hide it
        UserServiceError::EmailAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorBody::new("Email or username already exists")),
        ),
        UserServiceError::UsernameAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorBody::new("Email or username already exists")),
        ),
        UserServiceError::DatabaseError(err) => {
            tracing::error!("Datahase error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("internal server error")),
            )
        }
        UserServiceError::HashingError(err) => {
            tracing::error!("Hashing error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("internal server error")),
            )
        }
        other_error => {
            tracing::error!("Unexpected error {:?}", other_error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("internal server error")),
            )
        }
    })?;

    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<ErrorBody>)> {
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
                tracing::error!("Hasing error {:?}", err);
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
                tracing::error!("Datahase error: {:?}", err);
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
