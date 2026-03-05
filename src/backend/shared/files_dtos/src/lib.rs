use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
