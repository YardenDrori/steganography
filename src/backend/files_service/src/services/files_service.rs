use s3::{request::ResponseData, serde_types::Part, Bucket};
use sqlx::PgPool;

use crate::{
    dtos::{
        CompleteRequest, DownloadResponse, FileResponse, InitiateResponse, PartInfo,
        UploadPartResponse,
    },
    errors::files_service_errors::FilesServiceError,
    models::file::File,
    repositories::{
        self,
        files_repository::{self, create_file, get_file_by_id, list_file_by_user_id},
    },
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

pub async fn list_files(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<FileResponse>, FilesServiceError> {
    let files = list_file_by_user_id(pool, user_id)
        .await
        .map_err(|e| FilesServiceError::DatabaseError(e))?;
    Ok(files
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<FileResponse>>())
}

pub async fn get_download_url(
    bucket: &Bucket,
    pool: &PgPool,
    requesting_user_id: i64,
    is_admin: bool,
    file_id: i64,
) -> Result<DownloadResponse, FilesServiceError> {
    let file = validate_ownership(pool, requesting_user_id, is_admin, file_id).await?;
    Ok(DownloadResponse {
        download_url: bucket
            .presign_get(file.object_key(), 3600, None)
            .await
            .map_err(|e| FilesServiceError::StorageError(e))?
            .to_string(),
    })
}

pub async fn delete_file(
    bucket: &Bucket,
    pool: &PgPool,
    file_id: i64,
    requesting_user_id: i64,
    is_admin: bool,
) -> Result<(), FilesServiceError> {
    let file = validate_ownership(pool, requesting_user_id, is_admin, file_id).await?;

    bucket
        .delete_object(file.object_key())
        .await
        .map_err(|e| FilesServiceError::StorageError(e))?;
    files_repository::delete_file(pool, file_id)
        .await
        .map_err(|e| FilesServiceError::DatabaseError(e))?;
    Ok(())
}

pub async fn rename_file(
    pool: &PgPool,
    file_id: i64,
    requesting_user_id: i64,
    is_admin: bool,
    new_name: &str,
) -> Result<(), FilesServiceError> {
    validate_ownership(pool, requesting_user_id, is_admin, file_id).await?;
    repositories::files_repository::update_file_name(pool, file_id, &new_name)
        .await
        .map_err(|e| FilesServiceError::DatabaseError(e))?;
    Ok(())
}

pub async fn validate_ownership(
    pool: &PgPool,
    requesting_user_id: i64,
    is_admin: bool,
    file_id: i64,
) -> Result<File, FilesServiceError> {
    let file = get_file_by_id(pool, file_id)
        .await
        .map_err(|e| FilesServiceError::DatabaseError(e))?
        .ok_or(FilesServiceError::NotFound)?;
    if !is_admin && requesting_user_id != file.user_id() {
        return Err(FilesServiceError::Unauthorized);
    }
    Ok(file)
}
