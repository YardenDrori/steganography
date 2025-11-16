use crate::dtos::{UpdateUserRequest, UserCreateRequest, UserResponse};
use crate::errors::user_service_errors::UserServiceError;
use crate::repositories::user_repository;
use sqlx::PgPool;

pub async fn get_user(pool: &PgPool, user_id: i64) -> Result<UserResponse, UserServiceError> {
    let user = user_repository::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or(UserServiceError::NotFound)?;

    Ok(user.into())
}

pub async fn create_user(
    pool: &PgPool,
    request: &UserCreateRequest,
) -> Result<UserResponse, UserServiceError> {
    let user = user_repository::create_user(
        pool,
        &request.user_name,
        &request.first_name,
        &request.last_name,
        request.is_male,
        &request.email,
        request.phone_number.as_deref(),
    )
    .await
    .map_err(|e| UserServiceError::DatabaseError(e))?;
    Ok(user.into())
}

pub async fn update_user(
    pool: &PgPool,
    user_id: i64,
    request: &UpdateUserRequest,
) -> Result<UserResponse, UserServiceError> {
    tracing::info!(user_id = %user_id, "Updating user profile");

    // Check if user exists
    let existing_user = user_repository::get_user_by_id(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or(UserServiceError::NotFound)?;

    // Update user
    let updated_user = user_repository::update_user(
        pool,
        user_id,
        request.first_name.as_deref(),
        request.last_name.as_deref(),
        request.email.as_deref(),
        request.phone_number.as_deref(),
        request.is_male,
    )
    .await
    .map_err(|e| UserServiceError::DatabaseError(e))?;

    tracing::info!(user_id = %user_id, "User profile updated");
    Ok(updated_user.into())
}

pub async fn delete_user(pool: &PgPool, user_id: i64) -> Result<(), UserServiceError> {
    let deleted = user_repository::delete_user(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    if !deleted {
        return Err(UserServiceError::NotFound);
    }

    Ok(())
}
