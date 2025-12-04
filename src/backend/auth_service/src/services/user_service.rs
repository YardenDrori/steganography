use crate::dtos::LoginResponse;
use crate::errors::user_service_error::{self, UserServiceError};
use crate::models::user::User;
use crate::repositories::user_repository::{self, get_user_by_email, get_user_by_username};
use crate::services::token_service;
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
    tracing::info!(
        user_name = %user_name,
        email = %email,
        "Starting user registration saga"
    );

    //Step 1 save user data in user_service
    tracing::debug!(
        user_service_url = %user_service_url,
        "Saga step 1: Creating user profile in user_service"
    );
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
        let status = response.status();

        // Handle 409 Conflict (duplicate email/username)
        if status == reqwest::StatusCode::CONFLICT {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown conflict".to_string());

            tracing::warn!(
                user_name = %user_name,
                email = %email,
                error = %error_body,
                "user_service reported conflict"
            );

            // Parse error message to determine which field conflicted
            if error_body.to_lowercase().contains("email") {
                return Err(UserServiceError::EmailAlreadyExists);
            } else if error_body.to_lowercase().contains("username") {
                return Err(UserServiceError::UsernameAlreadyExists);
            } else {
                return Err(UserServiceError::ExternalServiceError(format!(
                    "Conflict: {}",
                    error_body
                )));
            }
        }

        return Err(UserServiceError::ExternalServiceError(format!(
            "user_service request failed: {}",
            status
        )));
    }

    let user_profile: UserResponse = response.json().await.map_err(|e| {
        tracing::error!(error = ?e, "Failed to parse user_service response");
        UserServiceError::ExternalServiceError(format!("invalid response frim user_service: {}", e))
    })?;

    let user_id = user_profile.id;
    tracing::info!(
        user_id = %user_id,
        user_name = %user_name,
        "Saga step 1: User profile created in user_service"
    );

    //Step 2 save user data in auth_service
    tracing::debug!(
        user_id = %user_id,
        "Saga step 2: Creating auth credentials in auth_service"
    );
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
                tracing::warn!(
                    user_id = %user_id,
                    error = ?e,
                    "Saga step 2: Failed to create auth credentials, initiating compensation"
                );
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

    tracing::info!(
        user_id = %user_id,
        auth_user_id = %user.id(),
        "Saga step 2: Auth credentials created"
    );

    // Assign default "user" role to new user
    tracing::debug!(user_id = %user_id, "Saga step 3: Assigning default user role");
    match user_repository::add_user_role(pool, user.id(), Role::User).await {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Saga step 3: Failed to assign role, initiating compensation"
            );
            compensate_delete_user(&user_service_url, &internal_api_key, user_id).await?;
            return Err(UserServiceError::DatabaseError(e));
        }
    }

    tracing::info!(
        user_id = %user_id,
        user_name = %user_name,
        email = %email,
        "Registration saga completed successfully"
    );

    // Return the full user profile from user_service
    Ok(user_profile)
}

pub async fn login_user(
    pool: &PgPool,
    internal_api_key: &str,
    user_service_url: &str,
    email: Option<&str>,
    user_name: Option<&str>,
    password: &str,
    device_info: Option<&str>,
    jwt_private_key: &str,
) -> Result<LoginResponse, UserServiceError> {
    tracing::info!(
        email = ?email,
        user_name = ?user_name,
        device_info = ?device_info,
        "Login attempt"
    );

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
    let user = user.ok_or_else(|| {
        tracing::warn!(email = ?email, user_name = ?user_name, "Login failed: user not found");
        UserServiceError::InvalidCredentials
    })?;

    tracing::debug!(user_id = %user.id(), "User found, verifying password");

    if !user
        .verify_password(&password)
        .map_err(|e| UserServiceError::HashingError(e))?
    {
        tracing::warn!(user_id = %user.id(), "Login failed: invalid password");
        return Err(UserServiceError::InvalidCredentials);
    }

    // Check if user account is active
    if !user.is_active() {
        tracing::warn!(user_id = %user.id(), "Login failed: user account is deactivated");
        return Err(UserServiceError::InvalidCredentials);
    }

    tracing::debug!(user_id = %user.id(), "Password verified, creating tokens");
    let jwt_token = token_service::create_access_token(user.id(), pool, jwt_private_key).await?;
    let refresh_token =
        token_service::create_refresh_token(pool, user.id(), device_info.map(|s| s.to_string()))
            .await?;

    // Fetch full user profile from user_service
    tracing::debug!(user_id = %user.id(), "Fetching full user profile from user_service");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/users/{}", user_service_url, user.id()))
        .header("X-Internal-API-Key", internal_api_key)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user profile from user_service: {:?}", e);
            UserServiceError::ExternalServiceError(format!("{}", e))
        })?;

    if !response.status().is_success() {
        return Err(UserServiceError::ExternalServiceError(format!(
            "user_service request failed: {}",
            response.status()
        )));
    }

    let user_profile: UserResponse = response.json().await.map_err(|e| {
        UserServiceError::ExternalServiceError(format!("Invalid response from user_service: {}", e))
    })?;

    let response = LoginResponse {
        user: user_profile,
        access_token: jwt_token,
        refresh_token,
    };

    tracing::info!(user_id = %user.id(), "Login successful");

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

pub async fn compensate_delete_user(
    user_service_url: &str,
    internal_api_key: &str,
    user_id: i64,
) -> Result<(), UserServiceError> {
    const ATTEMPTS: u8 = 3;

    tracing::warn!(
        user_id = %user_id,
        attempts = ATTEMPTS,
        "Starting compensation: deleting user from user_service"
    );

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
            tracing::info!(
                user_id = %user_id,
                "Compensation successful: user deleted from user_service"
            );
            return Ok(());
        }
    }
    tracing::error!(
        user_id = %user_id,
        attempts = ATTEMPTS,
        errors = %errors,
        "Compensation failed: could not delete user from user_service after {} attempts",
        ATTEMPTS
    );
    return Err(UserServiceError::ExternalServiceError(format!(
        "{:?}",
        errors
    )));
}

/// Helper function to sync user active status to user_service
async fn sync_user_status_to_user_service(
    user_service_url: &str,
    internal_api_key: &str,
    user_id: i64,
    is_active: bool,
) -> Result<(), UserServiceError> {
    tracing::debug!(
        user_id = %user_id,
        is_active = %is_active,
        "Syncing user status to user_service"
    );

    let client = reqwest::Client::new();
    let sync_request = serde_json::json!({
        "is_active": is_active
    });

    let response = client
        .patch(format!(
            "{}/internal/users/{}/status",
            user_service_url, user_id
        ))
        .header("X-Internal-API-Key", internal_api_key)
        .json(&sync_request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to sync to user_service: {:?}", e);
            UserServiceError::ExternalServiceError(format!("{}", e))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body".to_string());
        tracing::error!(
            "user_service sync failed: status={}, error={}",
            status,
            error_body
        );
        return Err(UserServiceError::ExternalServiceError(format!(
            "user_service sync failed: {}",
            status
        )));
    }

    tracing::debug!(
        user_id = %user_id,
        is_active = %is_active,
        "Successfully synced to user_service"
    );

    Ok(())
}

pub async fn deactivate_user(
    pool: &PgPool,
    internal_api_key: &str,
    user_service_url: &str,
    user_id: i64,
) -> Result<(), UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Starting user deactivation saga"
    );

    // Step 1: Sync to user_service FIRST (before we change anything in auth_service)
    tracing::debug!(
        user_id = %user_id,
        "Saga step 1: Syncing deactivation to user_service"
    );
    sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, false).await?;

    tracing::info!(
        user_id = %user_id,
        "Saga step 1: user_service updated successfully"
    );

    // Step 2: Update auth_service database (with compensation if this fails)
    tracing::debug!(
        user_id = %user_id,
        "Saga step 2: Deactivating user in auth_service"
    );

    match user_repository::set_user_active_status(pool, user_id, false).await {
        Ok(_) => {
            tracing::info!(
                user_id = %user_id,
                "Saga step 2: auth_service updated successfully"
            );
        }
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Saga step 2: Failed to update auth_service, initiating compensation"
            );

            // COMPENSATION: Revert user_service back to active
            if let Err(comp_err) =
                sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, true)
                    .await
            {
                tracing::error!(
                    user_id = %user_id,
                    compensation_error = ?comp_err,
                    original_error = ?e,
                    "CRITICAL: Compensation failed! user_service is deactivated but auth_service update failed"
                );
            } else {
                tracing::info!(
                    user_id = %user_id,
                    "Compensation successful: reverted user_service to active"
                );
            }

            return Err(UserServiceError::DatabaseError(e));
        }
    }

    // Step 3: Revoke all refresh tokens (best effort - if this fails, tokens expire anyway)
    tracing::debug!(
        user_id = %user_id,
        "Saga step 3: Revoking all refresh tokens"
    );

    match token_service::revoke_all_user_tokens(pool, user_id).await {
        Ok(tokens_revoked) => {
            tracing::info!(
                user_id = %user_id,
                tokens_revoked = %tokens_revoked,
                "Saga step 3: Revoked all refresh tokens"
            );
        }
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Saga step 3: Failed to revoke tokens (non-critical - they expire in 14 days)"
            );
            // Don't fail the whole operation - user is already deactivated
            // Tokens will expire naturally
        }
    }

    tracing::info!(
        user_id = %user_id,
        "Deactivation saga completed successfully"
    );

    Ok(())
}

pub async fn activate_user(
    pool: &PgPool,
    internal_api_key: &str,
    user_service_url: &str,
    user_id: i64,
) -> Result<(), UserServiceError> {
    tracing::info!(
        user_id = %user_id,
        "Starting user activation saga"
    );

    // Step 1: Sync to user_service FIRST (before we change anything in auth_service)
    tracing::debug!(
        user_id = %user_id,
        "Saga step 1: Syncing activation to user_service"
    );
    sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, true).await?;

    tracing::info!(
        user_id = %user_id,
        "Saga step 1: user_service updated successfully"
    );

    // Step 2: Update auth_service database (with compensation if this fails)
    tracing::debug!(
        user_id = %user_id,
        "Saga step 2: Activating user in auth_service"
    );

    match user_repository::set_user_active_status(pool, user_id, true).await {
        Ok(_) => {
            tracing::info!(
                user_id = %user_id,
                "Saga step 2: auth_service updated successfully"
            );
        }
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Saga step 2: Failed to update auth_service, initiating compensation"
            );

            // COMPENSATION: Revert user_service back to inactive
            if let Err(comp_err) =
                sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, false)
                    .await
            {
                tracing::error!(
                    user_id = %user_id,
                    compensation_error = ?comp_err,
                    original_error = ?e,
                    "CRITICAL: Compensation failed! user_service is activated but auth_service update failed"
                );
            } else {
                tracing::info!(
                    user_id = %user_id,
                    "Compensation successful: reverted user_service to inactive"
                );
            }

            return Err(UserServiceError::DatabaseError(e));
        }
    }

    tracing::info!(
        user_id = %user_id,
        "Activation saga completed successfully"
    );

    Ok(())
}
