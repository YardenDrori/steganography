use crate::dtos::LoginResponse;
use crate::errors::user_service_error::{self, UserServiceError};
use crate::models::user::User;
use crate::repositories::user_repository;
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

    //Step 1 hash password
    tracing::debug!("Step 1: Hashing password");
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| UserServiceError::HashingError(e))?
        .to_string();

    //Step 2 save user data in user_service
    tracing::debug!(
        user_service_url = %user_service_url,
        "Step 2: Creating user profile in user_service"
    );
    let client = reqwest::Client::new();
    let user_create_request = serde_json::json!({
    "user_name": user_name,
    "first_name": first_name,
    "last_name": last_name,
    "email": email,
    "phone_number": phone_number,
    "is_male": is_male,
    "password_hash": hashed_password,
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
        "Step 2: User profile created in user_service"
    );

    //Step 3 assign default user role in auth_service
    tracing::debug!(user_id = %user_id, "Step 3: Assigning default user role");
    match user_repository::add_user_role(pool, user_id, Role::User).await {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Step 3: Failed to assign role, initiating compensation"
            );
            compensate_delete_user(&user_service_url, &internal_api_key, user_id).await?;
            return Err(UserServiceError::DatabaseError(e));
        }
    }

    tracing::info!(
        user_id = %user_id,
        user_name = %user_name,
        email = %email,
        "Registration completed successfully"
    );

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

    // Call user_service to verify credentials
    tracing::debug!("Calling user_service to verify credentials");
    let client = reqwest::Client::new();
    let verify_request = serde_json::json!({
        "email": email,
        "user_name": user_name,
        "password": password,
    });

    let response = client
        .post(format!("{}/internal/auth/verify-credentials", user_service_url))
        .header("X-Internal-API-Key", internal_api_key)
        .json(&verify_request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to call user_service verify-credentials: {:?}", e);
            UserServiceError::ExternalServiceError(format!("{}", e))
        })?;

    if !response.status().is_success() {
        tracing::warn!(
            email = ?email,
            user_name = ?user_name,
            status = %response.status(),
            "Login failed: invalid credentials"
        );
        return Err(UserServiceError::InvalidCredentials);
    }

    let user_profile: UserResponse = response.json().await.map_err(|e| {
        UserServiceError::ExternalServiceError(format!("Invalid response from user_service: {}", e))
    })?;

    let user_id = user_profile.id;
    tracing::debug!(user_id = %user_id, "Credentials verified, creating tokens");
    let jwt_token = token_service::create_access_token(user_id, pool, jwt_private_key).await?;
    let refresh_token =
        token_service::create_refresh_token(pool, user_id, device_info.map(|s| s.to_string()))
            .await?;

    tracing::info!(user_id = %user_id, "Login successful");

    Ok(LoginResponse {
        user: user_profile,
        access_token: jwt_token,
        refresh_token,
    })
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
        "Starting user deactivation"
    );

    // Step 1: Deactivate in user_service
    tracing::debug!(
        user_id = %user_id,
        "Step 1: Deactivating user in user_service"
    );
    sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, false).await?;

    tracing::info!(
        user_id = %user_id,
        "Step 1: user_service updated successfully"
    );

    // Step 2: Revoke all refresh tokens (best effort)
    tracing::debug!(
        user_id = %user_id,
        "Step 2: Revoking all refresh tokens"
    );

    match token_service::revoke_all_user_tokens(pool, user_id).await {
        Ok(tokens_revoked) => {
            tracing::info!(
                user_id = %user_id,
                tokens_revoked = %tokens_revoked,
                "Step 2: Revoked all refresh tokens"
            );
        }
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = ?e,
                "Step 2: Failed to revoke tokens (non-critical - they expire in 14 days)"
            );
        }
    }

    tracing::info!(
        user_id = %user_id,
        "User deactivation completed successfully"
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
        "Activating user account"
    );

    sync_user_status_to_user_service(user_service_url, internal_api_key, user_id, true).await?;

    tracing::info!(
        user_id = %user_id,
        "User activation completed successfully"
    );

    Ok(())
}
