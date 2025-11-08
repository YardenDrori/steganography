use crate::entities::token::RefreshTokenEntity;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RefreshToken {
    id: i64,
    user_id: i64,
    token_hash: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    device_info: Option<String>,
}

impl From<RefreshTokenEntity> for RefreshToken {
    fn from(entity: RefreshTokenEntity) -> Self {
        RefreshToken {
            id: entity.id,
            user_id: entity.user_id,
            token_hash: entity.token_hash,
            expires_at: entity.expires_at,
            created_at: entity.created_at,
            revoked_at: entity.revoked_at,
            device_info: entity.device_info,
        }
    }
}
