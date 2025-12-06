use crate::dtos::{UpdateUserRequest, UserCreateRequest, UserResponse, VerifyCredentialsRequest};
use crate::errors::user_service_errors::UserServiceError;
use crate::repositories::user_repository;
use sqlx::PgPool;

pub async fn get_user(pool: &PgPool, user_id: i64) -> Result<UserResponse, UserServiceError> {
    let user = user_repository::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = ?e,
                "Database error while fetching user"
            );
            UserServiceError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            tracing::warn!(
                user_id = %user_id,
                "User not found"
            );
            UserServiceError::NotFound
        })?;

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
        &request.password_hash,
    )
    .await
    .map_err(|e| {
        // Check for unique constraint violations
        match &e {
            sqlx::Error::Database(db_err) => {
                if let Some(constraint_name) = db_err.constraint() {
                    if constraint_name.contains("email") {
                        tracing::warn!(
                            email = %request.email,
                            "Attempted to create user with existing email"
                        );
                        return UserServiceError::EmailAlreadyExists;
                    } else if constraint_name.contains("user_name") {
                        tracing::warn!(
                            user_name = %request.user_name,
                            "Attempted to create user with existing username"
                        );
                        return UserServiceError::UsernameAlreadyExists;
                    }
                }
            }
            _ => {}
        }
        UserServiceError::DatabaseError(e)
    })?;
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
    tracing::info!(
        user_id = %user_id,
        "Deleting user"
    );

    let deleted = user_repository::delete_user(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = ?e,
                "Database error while deleting user"
            );
            UserServiceError::DatabaseError(e)
        })?;

    if !deleted {
        tracing::warn!(
            user_id = %user_id,
            "User not found for deletion"
        );
        return Err(UserServiceError::NotFound);
    }

    tracing::info!(
        user_id = %user_id,
        "User deleted successfully"
    );

    Ok(())
}

pub async fn set_user_active_status(
    pool: &PgPool,
    user_id: i64,
    is_active: bool,
) -> Result<UserResponse, UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        is_active = %is_active,
        "Syncing user active status"
    );

    let user = user_repository::set_user_active_status(pool, user_id, is_active)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserServiceError::NotFound,
            other => UserServiceError::DatabaseError(other),
        })?;

    tracing::info!(
        user_id = %user_id,
        is_active = %is_active,
        "User active status synced successfully"
    );

    Ok(user.into())
}

pub async fn verify_credentials(
    pool: &PgPool,
    request: &VerifyCredentialsRequest,
) -> Result<UserResponse, UserServiceError> {
    let user = user_repository::get_user_by_email_or_username(
        pool,
        request.email.as_deref(),
        request.user_name.as_deref(),
    )
    .await
    .map_err(|e| UserServiceError::DatabaseError(e))?
    .ok_or(UserServiceError::InvalidCredentials)?;

    if !user.is_active() {
        return Err(UserServiceError::Unauthorized);
    }

    let password_valid = user
        .verify_password(&request.password)
        .map_err(|_| UserServiceError::InvalidCredentials)?;

    if !password_valid {
        return Err(UserServiceError::InvalidCredentials);
    }

    Ok(user.into())
}
