use s3::Bucket;

use crate::{dtos::InitiateResponse, errors::files_service_errors::FilesServiceError};

pub async fn initiate_upload(bucket: &Bucket) -> Result<InitiateResponse, FilesServiceError> {
    let object_key = uuid::Uuid::new_v4().to_string();
    let response = bucket
        .initiate_multipart_upload(&object_key, "application/octet-stream")
        .await
        .map_err(|e| FilesServiceError::StorageError(e))?;
    Ok(InitiateResponse {
        upload_id: response.upload_id,
        object_key,
    })
}
