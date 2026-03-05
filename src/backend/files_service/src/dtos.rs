use crate::models::file::File;

pub use files_dtos::{CompleteRequest, FileResponse, InitiateResponse, PartInfo, UploadPartResponse};

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

