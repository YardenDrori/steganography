use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct FileEntity {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub object_key: String,
    pub created_at: DateTime<Utc>,
    pub is_carrier: bool,
    pub is_steg_object: bool,
}
