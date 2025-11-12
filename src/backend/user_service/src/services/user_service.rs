use crate::dtos::UserResponse;
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
