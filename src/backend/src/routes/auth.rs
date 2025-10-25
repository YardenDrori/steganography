use crate::errors::error_body::ErrorBody;
use crate::errors::user_service_error::UserServiceError;
use crate::models::user::{RegisterRequest, UserResponse};
use crate::services::user_service;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::MySqlPool;

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_response = user_service::register_user(&pool, payload)
        .await
        .map_err(|e| match e {
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
        })?;

    Ok((StatusCode::CREATED, Json(user_response)))
}
