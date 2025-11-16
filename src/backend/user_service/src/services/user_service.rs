use crate::dtos::{UserCreateRequest, UserResponse};
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

pub async fn delete_user(pool: &PgPool, user_id: i64) -> Result<(), UserServiceError> {
    let deleted = user_repository::delete_user(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    if !deleted {
        return Err(UserServiceError::NotFound);
    }

    Ok(())
}
