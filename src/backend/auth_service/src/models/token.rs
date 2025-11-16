use crate::entities::token::RefreshTokenEntity;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct RefreshToken {
    id: i64,
    user_id: i64,
    token_hash: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    device_info: Option<String>,
}

impl RefreshToken {
    // Getters
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn device_info(&self) -> Option<&str> {
        self.device_info.as_deref()
    }

    // Domain methods
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn matches_token(&self, token_to_verify: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(token_to_verify.as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());
        computed_hash == self.token_hash
    }
}

impl From<RefreshTokenEntity> for RefreshToken {
    fn from(entity: RefreshTokenEntity) -> Self {
        RefreshToken {
            id: entity.id,
            user_id: entity.user_id,
            token_hash: entity.token_hash,
            expires_at: entity.expires_at,
            created_at: entity.created_at,
            device_info: entity.device_info,
        }
    }
}
