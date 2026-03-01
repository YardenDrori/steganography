use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::file::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitiateResponse {
    pub upload_id: String,
    pub object_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartInfo {
    pub part_number: u32,
    pub etag: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UploadPartResponse {
    pub part: PartInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompleteRequest {
    pub upload_id: String,
    pub object_key: String,
    pub filename: String,
    pub parts: Vec<PartInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResponse {
    pub id: i64,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        FileResponse {
            id: file.id(),
            filename: file.filename().to_string(),
            created_at: file.created_at(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameFileRequest {
    pub new_name: String,
}

