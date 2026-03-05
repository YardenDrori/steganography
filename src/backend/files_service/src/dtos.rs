use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::file::File;

pub use files_dtos::{CompleteRequest, InitiateResponse, PartInfo, UploadPartResponse};

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

