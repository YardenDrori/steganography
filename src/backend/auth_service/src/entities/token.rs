use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub(crate) struct RefreshTokenEntity {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub device_info: Option<String>,
}
