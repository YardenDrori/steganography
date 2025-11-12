use crate::dtos::UserResponse;
use crate::errors::user_service_errors::UserServiceError;
use crate::repositories::user_repository;
use shared_global::auth::roles::Roles;
use sqlx::PgPool;

pub async fn get_user(pool: &PgPool, user_id: i64) -> Result<UserResponse, UserServiceError> {
    let user = user_repository::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or(UserServiceError::NotFound)?;

    Ok(user.into())
}

pub async fn get_user_roles(pool: &PgPool, user_id: i64) -> Result<Roles, UserServiceError> {
    user_repository::get_user_roles(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))
}
