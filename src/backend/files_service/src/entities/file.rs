use chrono::{DateTime, Utc};

use crate::models::file::File;

#[derive(Debug, Clone)]
pub struct FileEntity {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub object_key: String,
    pub created_at: DateTime<Utc>,
}
