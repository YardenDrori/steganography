use std::slice::ChunkBy;

use s3::{bucket, request::ResponseData, serde_types::Part, Bucket};
use sqlx::{pool, PgPool};

use crate::{
    dtos::{CompleteRequest, FileResponse, InitiateResponse, PartInfo, UploadPartResponse},
    errors::files_service_errors::FilesServiceError,
    repositories::files_repository::create_file,
};

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

pub async fn upload_chunk(
    bucket: &Bucket,
    chunk: Vec<u8>,
    path: &str,
    part_number: u32,
    upload_id: &str,
) -> Result<UploadPartResponse, FilesServiceError> {
    let part: Part = bucket
        .put_multipart_chunk(
            chunk,
            path,
            part_number,
            upload_id,
            "application/octet-stream",
        )
        .await
        .map_err(|e| FilesServiceError::StorageError(e))?;
    Ok(UploadPartResponse {
        part: PartInfo {
            etag: part.etag,
            part_number: part.part_number,
        },
    })
}

pub async fn complete_upload(
    bucket: &Bucket,
    pool: &PgPool,
    complete_request: CompleteRequest,
    uploader_id: i64,
) -> Result<FileResponse, FilesServiceError> {
    let _response: ResponseData = bucket
        .complete_multipart_upload(
            &complete_request.object_key,
            &complete_request.upload_id,
            complete_request
                .parts
                .into_iter()
                .map(|i| Part {
                    part_number: i.part_number,
                    etag: i.etag,
                })
                .collect(),
        )
        .await
        .map_err(|e| FilesServiceError::StorageError(e))?;

    let file_response = create_file(
        pool,
        uploader_id,
        &complete_request.filename,
        &complete_request.object_key,
    )
    .await
    .map_err(|e| FilesServiceError::DatabaseError(e))?;

    Ok(FileResponse {
        id: file_response.id(),
        filename: file_response.filename().to_string(),
        created_at: file_response.created_at(),
    })
}
