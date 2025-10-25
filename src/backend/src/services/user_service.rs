use crate::errors::user_service_error::{self, UserServiceError};
use crate::models::user::{self};
use crate::repositories::user_repository;
use bcrypt::{DEFAULT_COST, hash};
use sqlx::MySqlPool;

pub async fn register_user(
    pool: &MySqlPool,
    request: user::RegisterRequest,
) -> Result<user::UserResponse, user_service_error::UserServiceError> {
    //check if user exists
    if user_repository::get_user_by_email(pool, &request.email)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .is_some()
    {
        return Err(UserServiceError::EmailAlreadyExists);
    }
    if user_repository::get_user_by_username(pool, &request.user_name)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .is_some()
    {
        return Err(UserServiceError::UsernameAlreadyExists);
    }

    let hashed_password =
        hash(request.password, DEFAULT_COST).map_err(|e| UserServiceError::HashingError(e))?;

    let user = user_repository::create_user(
        pool,
        &request.user_name,
        &request.first_name,
        &request.last_name,
        &request.email,
        &hashed_password,
        &request.phone_number.as_deref(),
        &request.is_male,
    )
    .await
    .map_err(|e| UserServiceError::DatabaseError(e))?;

    let response = user::UserResponse {
        id: user.id,
        user_name: user.user_name,
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        phone_number: user.phone_number,
        is_male: user.is_male,
        created_at: user.created_at,
        is_active: user.is_active,
        is_verified: user.is_verified,
    };

    Ok(response)
}
