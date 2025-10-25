use crate::errors::error_body::ErrorBody;
use crate::errors::user_service_error::UserServiceError;
use crate::models::user::{LoginRequest, LoginResponse, RegisterRequest, UserResponse};
use crate::services::user_service::{login_user, register_user};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::MySqlPool;

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_response = register_user(&pool, payload).await.map_err(|e| match e {
        UserServiceError::EmailAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorBody {
                error_message: "Email or username already exists".to_string(),
            }),
        ),
        UserServiceError::UsernameAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorBody {
                error_message: "Email or username already exists".to_string(),
            }),
        ),
        UserServiceError::DatabaseError(err) => {
            tracing::error!("Datahase error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error_message: "Internal server error".to_string(),
                }),
            )
        }
        UserServiceError::HashingError(err) => {
            tracing::error!("Hashing error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error_message: "Internal server error".to_string(),
                }),
            )
        }
        other_error => {
            tracing::error!("Unexpected error {:?}", other_error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error_message: "Internal server error".to_string(),
                }),
            )
        }
    })?;

    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<ErrorBody>)> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("database secret must be set in .env file");

    let login_response = login_user(&pool, payload, &jwt_secret)
        .await
        .map_err(|e| match e {
            UserServiceError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error_message: "Invalid credentials".to_string(),
                }),
            ),
            UserServiceError::HashingError(err) => {
                tracing::error!("Hasing error {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error_message: "Internal server error".to_string(),
                    }),
                )
            }
            UserServiceError::JwtError(err) => {
                tracing::error!("user service error {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error_message: "Internal server error".to_string(),
                    }),
                )
            }
            UserServiceError::DatabaseError(err) => {
                tracing::error!("Datahase error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error_message: "Internal server error".to_string(),
                    }),
                )
            }
            other_error => {
                tracing::error!("Unexpected error {:?}", other_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error_message: "Internal server error".to_string(),
                    }),
                )
            }
        })?;

    Ok((StatusCode::OK, Json(login_response)))
}
