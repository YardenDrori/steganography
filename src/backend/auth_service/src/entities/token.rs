use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub(crate) struct RefreshTokenEntity {
    id: i64,
    user_id: i64,
    token_hash: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    device_info: String,
}
