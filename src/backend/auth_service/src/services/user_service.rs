use std::str::FromStr;

use crate::dtos::LoginResponse;
use crate::errors::user_service_error::{self, UserServiceError};
use crate::models::user::User;
use crate::repositories::user_repository::{self, get_user_by_email, get_user_by_username};
use crate::services::token_service::{create_access_token, create_refresh_token};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use shared_global::auth::roles::{Role, Roles};
use shared_global::dtos::UserResponse;
use sqlx::PgPool;

pub async fn register_user(
    pool: &PgPool,
    user_name: &str,
    email: &str,
    password: &str,
) -> Result<UserResponse, user_service_error::UserServiceError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| UserServiceError::HashingError(e))?
        .to_string();

    let user = user_repository::save_user(pool, &user_name, &email, &hashed_password)
        .await
        .map_err(|e| {
            // Check if this is a database-specific error (not network, timeout, etc.)
            match &e {
                sqlx::Error::Database(db_err) => {
                    // Check if the error has a constraint name (means unique/foreign key violation)
                    match db_err.constraint() {
                        Some(constraint_name) => {
                            // Check which constraint was violated
                            if constraint_name.contains("email") {
                                UserServiceError::EmailAlreadyExists
                            } else if constraint_name.contains("user_name") {
                                UserServiceError::UsernameAlreadyExists
                            } else {
                                // Some other constraint we don't handle specifically
                                UserServiceError::DatabaseError(e)
                            }
                        }
                        None => {
                            // Database error but no constraint (e.g., connection issue)
                            UserServiceError::DatabaseError(e)
                        }
                    }
                }
                _ => {
                    // Not a database error (could be network, timeout, etc.)
                    UserServiceError::DatabaseError(e)
                }
            }
        })?;

    // Assign default "user" role to new user
    user_repository::add_user_role(pool, user.id(), Role::User)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    let response = user_to_response(user);
    Ok(response)
}

pub async fn login_user(
    pool: &PgPool,
    email: Option<&str>,
    user_name: Option<&str>,
    password: &str,
    device_info: Option<&str>,
    jwt_secret: &str,
) -> Result<LoginResponse, UserServiceError> {
    let user = match (email, user_name) {
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
        .verify_password(&password)
        .map_err(|e| UserServiceError::HashingError(e))?
    {
        return Err(UserServiceError::InvalidCredentials);
    }

    let jwt_token = create_access_token(user.id(), pool, jwt_secret).await?;
    let refresh_token =
        create_refresh_token(pool, user.id(), device_info.map(|s| s.to_string())).await?;

    let response = user_to_response(user);

    let response = LoginResponse {
        user: response,
        access_token: jwt_token,
        refresh_token,
    };

    Ok(response)
}

pub async fn get_user_roles(pool: &PgPool, user_id: i64) -> Result<Roles, sqlx::Error> {
    let records = sqlx::query!(
        r#"
        SELECT role
        FROM user_roles
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut roles = Roles::new(); // Initialize!
    for record in records {
        if let Ok(role) = Role::from_str(&record.role) {
            roles.insert(role);
        }
    }

    Ok(roles)
}

fn user_to_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id(),
        user_name: user.user_name().to_string(),
        email: user.email().to_string(),
        is_active: user.is_active(),
        is_verified: user.is_verified(),
    }
}
