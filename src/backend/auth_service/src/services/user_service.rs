use crate::auth::jwt::create_jwt;
use crate::dtos::{LoginRequest, LoginResponse, RegisterRequest};
use crate::errors::user_service_error::{self, UserServiceError};
use crate::models::user::User;
use crate::repositories::user_repository::{self, get_user_by_email, get_user_by_username};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};
use shared::dtos::UserResponse;
use sqlx::PgPool;

pub async fn register_user(
    pool: &PgPool,
    request: RegisterRequest,
) -> Result<UserResponse, user_service_error::UserServiceError> {
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

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|e| UserServiceError::HashingError(e))?
        .to_string();

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

    let response = user_to_response(user);
    Ok(response)
}

pub async fn login_user(
    pool: &PgPool,
    request: LoginRequest,
    jwt_secret: &str,
) -> Result<LoginResponse, UserServiceError> {
    let user = match (request.email, request.user_name) {
        (Some(email), None) => get_user_by_email(pool, &email)
            .await
            .map_err(|e| UserServiceError::DatabaseError(e))?,
        (None, Some(user_name)) => get_user_by_username(pool, &user_name)
            .await
            .map_err(|e| UserServiceError::DatabaseError(e))?,
        _ => {
            return Err(user_service_error::UserServiceError::InvalidCredentials);
        }
    };

    //dont tell users if an account for those credentials exist or not
    let user = user.ok_or(UserServiceError::InvalidCredentials)?;

    if !user
        .verify_password(&request.password)
        .map_err(|e| UserServiceError::HashingError(e))?
    {
        return Err(UserServiceError::InvalidCredentials);
    }

    let jwt_token = create_jwt(user.id(), jwt_secret).map_err(|e| UserServiceError::JwtError(e))?;

    let response = user_to_response(user);

    let response = LoginResponse {
        user: response,
        access_token: jwt_token,
    };

    Ok(response)
}

fn user_to_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id(),
        user_name: user.user_name().to_string(),
        first_name: user.first_name().to_string(),
        last_name: user.last_name().to_string(),
        email: user.email().to_string(),
        phone_number: user.phone_number().map(|s| s.to_string()),
        is_male: user.is_male(),
        created_at: user.created_at(),
        is_active: user.is_active(),
        is_verified: user.is_verified(),
    }
}
