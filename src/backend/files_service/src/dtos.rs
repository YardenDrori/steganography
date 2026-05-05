use crate::models::file::File;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use files_dtos::{
    CompleteRequest, FileResponse, InitiateResponse, PartInfo, UploadPartResponse,
};

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        FileResponse {
            id: file.id(),
            filename: file.filename().to_string(),
            created_at: file.created_at(),
            is_carrier: file.is_carrier(),
            is_steg_object: file.is_steg_object(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminFileResponse {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub created_at: DateTime<Utc>,
    pub is_carrier: bool,
    pub is_steg_object: bool,
}

impl From<File> for AdminFileResponse {
    fn from(file: File) -> Self {
        AdminFileResponse {
            id: file.id(),
            user_id: file.user_id(),
            filename: file.filename().to_string(),
            created_at: file.created_at(),
            is_carrier: file.is_carrier(),
            is_steg_object: file.is_steg_object(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameFileRequest {
    pub new_name: String,
}
