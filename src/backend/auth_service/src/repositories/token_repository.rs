use crate::entities::token::RefreshTokenEntity;
use crate::models::token::RefreshToken;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: i64,
    token_hash: &str,
    expires_at: DateTime<Utc>,
    device_info: &str,
) -> Result<RefreshToken, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, device_info)
        VALUES ($1, $2, $3, $4) RETURNING id
        "#,
        user_id,
        token_hash,
        expires_at,
        device_info,
    )
    .fetch_one(pool)
    .await?;

    let refresh_token_id = result.id;
    let refresh_token = get_refrsh_token_by_id(pool, refresh_token_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(refresh_token)
}

pub async fn get_refrsh_token_by_id(
    pool: &PgPool,
    refresh_token_id: i64,
) -> Result<Option<RefreshToken>, sqlx::Error> {
    let refresh_token = sqlx::query_as!(
        RefreshTokenEntity,
        r#"
        SELECT id, user_id, token_hash, expires_at, created_at, revoked_at , device_info FROM refresh_tokens WHERE id = $1
        "#,
        refresh_token_id
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());

    Ok(refresh_token)
}
