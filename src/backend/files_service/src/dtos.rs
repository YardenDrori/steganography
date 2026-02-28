use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresignRequest {
    pub filename: String,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresignResponse {
    pub presigned_url: String,
    pub object_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfirmRequest {
    pub object_key: String,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResponse {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadResponse {
    pub download_url: String,
}
