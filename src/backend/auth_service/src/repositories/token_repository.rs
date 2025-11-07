use crate::entities::token::RefreshTokenEntity;
use crate::models::user::User;
use chrono::DateTime;
use sqlx::PgPool;


pub fn create_refresh_token(user_id: i64, token_hash: &str, expires_at: DateTime<Utc>,)
