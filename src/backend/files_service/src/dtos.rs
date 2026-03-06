use crate::models::file::File;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameFileRequest {
    pub new_name: String,
}
