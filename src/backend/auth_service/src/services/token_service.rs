use crate::auth::jwt::encode_jwt;
use crate::errors::user_service_error::UserServiceError;
use crate::models::token::RefreshToken;
use crate::repositories::{token_repository, user_repository};
use crate::services::user_service::get_user_roles;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use shared_global::auth::roles::Roles;
use sqlx::PgPool;

const ACCESS_TOKEN_DURATION_SECONDS: i64 = 10 * 60; // 10 minutes
const REFRESH_TOKEN_LENGTH: usize = 64;
const REFRESH_TOKEN_DURATION_DAYS: i64 = 14;
const MAX_COLLISION_ATTEMPTS: u8 = 3;

// Generates a cryptographically secure random refresh token
fn generate_random_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..REFRESH_TOKEN_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// Hashes a token using SHA256
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn create_access_token(
    user_id: i64,
    pool: &PgPool,
    private_key_pem: &str,
) -> Result<String, UserServiceError> {
    tracing::debug!("Creating access token for user_id={}", user_id);
    let now = Utc::now();
    let issued_at = now.timestamp();
    let expires_at = issued_at + ACCESS_TOKEN_DURATION_SECONDS;
    let roles = get_user_roles(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    let token = encode_jwt(user_id, issued_at, expires_at, roles, private_key_pem)
        .map_err(|e| UserServiceError::JwtError(e))?;
    tracing::info!("Created access token for user_id={}", user_id);
    Ok(token)
}

// Creates a new refresh token for a user
pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: i64,
    device_info: Option<String>,
) -> Result<String, UserServiceError> {
    tracing::debug!("Creating refresh token for user_id={}", user_id);
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);

    // Try to generate unique token (retry on collision)
    for attempt in 0..MAX_COLLISION_ATTEMPTS {
        let token = generate_random_token();
        let token_hash = hash_token(&token);

        match token_repository::save_refresh_token(
            pool,
            user_id,
            &token_hash,
            expires_at,
            device_info.as_deref().unwrap_or(""),
        )
        .await
        {
            Ok(_) => {
                tracing::info!("Created refresh token for user_id={}", user_id);
                return Ok(token); // Return plaintext token to send to client
            }
            Err(sqlx::Error::Database(db_err)) => {
                // Check if it's a unique constraint violation
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("token_hash") {
                        tracing::warn!(
                            "Token hash collision on attempt {}/{}",
                            attempt + 1,
                            MAX_COLLISION_ATTEMPTS
                        );
                        continue; // Try again
                    }
                }
                // Some other database error
                return Err(UserServiceError::DatabaseError(sqlx::Error::Database(
                    db_err,
                )));
            }
            Err(e) => return Err(UserServiceError::DatabaseError(e)),
        }
    }

    // Failed after all attempts
    tracing::error!(
        "Failed to create refresh token after {} attempts",
        MAX_COLLISION_ATTEMPTS
    );
    Err(UserServiceError::DatabaseError(sqlx::Error::RowNotFound))
}

pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token: &str,
    jwt_private_key: &str,
) -> Result<(String, String), UserServiceError> {
    tracing::debug!("Attempting to refresh access token");
    // Hash the provided token to look it up
    let token_hash = hash_token(refresh_token);

    // Find the token in database
    let stored_token = token_repository::get_refresh_token_by_hash(pool, &token_hash)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or_else(|| {
            tracing::warn!("Refresh token not found in database");
            UserServiceError::InvalidCredentials
        })?;

    // Validate the token
    if stored_token.is_expired() {
        tracing::warn!(
            "Expired refresh token used for user_id={}",
            stored_token.user_id()
        );
        // Delete expired token
        token_repository::revoke_refresh_token(pool, stored_token.id())
            .await
            .map_err(|e| UserServiceError::DatabaseError(e))?;
        return Err(UserServiceError::InvalidCredentials);
    }

    // Verify user still exists and is active
    let user = user_repository::get_user_by_id(pool, stored_token.user_id())
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or_else(|| {
            tracing::error!("User not found for refresh token");
            UserServiceError::InvalidCredentials
        })?;

    if !user.is_active() {
        tracing::warn!("Inactive user attempted to refresh token: {}", user.id());
        return Err(UserServiceError::InvalidCredentials);
    }

    // Revoke the old refresh token (token rotation)
    token_repository::revoke_refresh_token(pool, stored_token.id())
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    // Generate new access token
    let new_access_token = create_access_token(user.id(), &pool, jwt_private_key).await?;

    // Generate new refresh token
    let new_refresh_token = create_refresh_token(
        pool,
        user.id(),
        stored_token.device_info().map(|s| s.to_string()),
    )
    .await?;

    tracing::info!("Rotated tokens for user_id={}", user.id());

    Ok((new_access_token, new_refresh_token))
}

// Revokes a refresh token (for logout)
pub async fn revoke_refresh_token(
    pool: &PgPool,
    refresh_token: &str,
) -> Result<(), UserServiceError> {
    tracing::debug!("Attempting to revoke refresh token");
    let token_hash = hash_token(refresh_token);

    let stored_token = token_repository::get_refresh_token_by_hash(pool, &token_hash)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?
        .ok_or(UserServiceError::InvalidCredentials)?;

    token_repository::revoke_refresh_token(pool, stored_token.id())
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    tracing::info!(
        "Revoked refresh token for user_id={}",
        stored_token.user_id()
    );

    Ok(())
}

// Revokes all refresh tokens for a user (for account deactivation)
pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: i64) -> Result<u64, UserServiceError> {
    tracing::debug!("Attempting to revoke all tokens for user_id={}", user_id);

    let tokens_deleted = token_repository::revoke_all_user_tokens(pool, user_id)
        .await
        .map_err(|e| UserServiceError::DatabaseError(e))?;

    tracing::info!(
        "Revoked {} refresh token(s) for user_id={}",
        tokens_deleted,
        user_id
    );

    Ok(tokens_deleted)
}
