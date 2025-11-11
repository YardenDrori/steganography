use crate::errors::user_service_errors::UserServiceError;
use crate::models::user::User;
use crate::repositories::user_repository;
use sqlx::PgPool;

pub async fn get_user_by_id(pool: &PgPool, user_id: i64) -> Result<User, UserServiceError> {
    let user = user_repository::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or(UserServiceError::NotFound)?;
    Ok(user)
}
