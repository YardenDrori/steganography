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
use std::str::FromStr;

pub async fn register_user(
    pool: &PgPool,
    internal_api_key: &str,
    user_service_url: &str,
    user_name: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone_number: Option<&str>,
    is_male: Option<bool>,
    password: &str,
) -> Result<UserResponse, user_service_error::UserServiceError> {
    //Step 1 save user data in user_service
    let client = reqwest::Client::new();
    let user_create_request = serde_json::json!({
    "user_name": user_name,
    "first_name": first_name,
    "last_name": last_name,
    "email": email,
    "phone_number": phone_number,
    "is_male": is_male,
    });

    let response = client
        .post(format!("{}/users", user_service_url))
        .header("X-Internal-API-Key", internal_api_key)
        .json(&user_create_request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("failed to call user_service");
            UserServiceError::ExternalServiceError(format!("{}", e))
        })?;

    if !response.status().is_success() {
        return Err(UserServiceError::ExternalServiceError(format!(
            "user_service request failed: {}",
            response.status()
        )));
    }

    let user_profile: UserResponse = response.json().await.map_err(|e| {
        UserServiceError::ExternalServiceError(format!("invalid response frim user_service: {}", e))
    })?;

    let user_id = user_profile.id;
    tracing::info!("Created user profile in user_service with id={}", user_id);

    //Step 2 save user data in auth_service
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| UserServiceError::HashingError(e))?
        .to_string();

    let user: User =
        match user_repository::save_user(pool, &user_name, &email, &hashed_password).await {
            Ok(r) => r,
            Err(e) => {
                compensate_delete_user(user_service_url, internal_api_key, user_id).await?;

                match &e {
                    sqlx::Error::Database(db_err) => {
                        match db_err.constraint() {
                            Some(constraint_name) => {
                                if constraint_name.contains("email") {
                                    tracing::error!(
                                    "Attempted to create an account under an existing email: {}"
                                        ,email
                                );
                                    return Err(UserServiceError::EmailAlreadyExists);
                                } else if constraint_name.contains("user_name") {
                                    tracing::error!(
                                    "Attempted to create an account under an existing user name: {}"
                                        ,user_name
                                );
                                    return Err(UserServiceError::UsernameAlreadyExists);
                                } else {
                                    tracing::error!(
                                        "Unexpected constraint conflict: {}",
                                        constraint_name
                                    );
                                    return Err(UserServiceError::DatabaseError(e));
                                }
                            }
                            None => {
                                tracing::error!("Unexpected database error: {:?}", e);
                                return Err(UserServiceError::DatabaseError(e));
                            }
                        };
                    }
                    _ => {
                        tracing::error!("Unexpected database error: {:?}", e);
                        return Err(UserServiceError::DatabaseError(e));
                    }
                }
            }
        };

    // Assign default "user" role to new user
    match user_repository::add_user_role(pool, user.id(), Role::User).await {
        Ok(u) => u,
        Err(e) => {
            compensate_delete_user(&user_service_url, &internal_api_key, user_id).await?;
            return Err(UserServiceError::DatabaseError(e));
        }
    }

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

pub async fn compensate_delete_user(
    user_service_url: &str,
    internal_api_key: &str,
    user_id: i64,
) -> Result<(), UserServiceError> {
    const ATTEMPTS: u8 = 3;

    let client = reqwest::Client::new();
    let mut errors = String::new();
    for i in 0..ATTEMPTS {
        let response: reqwest::Response = match client
            .delete(format!("{}/users/{}", user_service_url, user_id))
            .header("X-Internal-API-Key", internal_api_key)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Attempt: {} out of {}: {:?}", i, ATTEMPTS, e);
                errors.push_str(&format!("Attempt: {} out of {}: {:?}", i, ATTEMPTS, e));
                continue;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            tracing::error!(
                "Attempt {} out of {}: status={}, error={}",
                i,
                ATTEMPTS,
                status,
                error_body
            );
            errors.push_str(&format!(
                "Attempt {} out of {}: status={}, error={}\n",
                i, ATTEMPTS, status, error_body
            ));
            continue;
        } else {
            return Ok(());
        }
    }
    tracing::info!("Deleted user profile in user_service with id={} due to auth_user db failing to create matching user data", user_id);
    return Err(UserServiceError::ExternalServiceError(format!(
        "{:?}",
        errors
    )));
}
