use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PrepareUploadRequest {
    #[validate(length(min = 1, max = 255, message = "Filename must be between 1 and 255 characters"))]
    pub filename: String,
    #[validate(length(min = 1, max = 100, message = "Content type must be between 1 and 100 characters"))]
    pub content_type: String,
}

#[derive(Debug, Serialize)]
pub struct PrepareUploadResponse {
    pub file_id: i64,
    pub upload_url: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: i64,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmUploadResponse {
    pub id: i64,
    pub filename: String,
    pub size_bytes: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteFileResponse {
    pub message: String,
}
