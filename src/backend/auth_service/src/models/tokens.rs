use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub(crate) struct RefreshTokenEntity {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub device_info: String
}
