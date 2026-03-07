use crate::errors::steg_service_error::StegServiceError;
use files_dtos::{CompleteRequest, FileResponse, InitiateResponse, PartInfo, UploadPartResponse};
use reqwest::{Response, StatusCode};
use std::path::PathBuf;
use tempfile;
use tokio::{
    fs::{File, remove_file},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

pub async fn download_file_to_temp(
    client: &reqwest::Client,
    files_service_url: &str,
    file_id: i64,
    bearer_token: &str,
) -> Result<(PathBuf, bool, String), StegServiceError> {
    // Fetch file metadata to check is_carrier
    let meta_response = client
        .get(format!("{}/files/{}", files_service_url, file_id))
        .bearer_auth(bearer_token)
        .send()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;

    if meta_response.status() != StatusCode::OK {
        return Err(StegServiceError::ExternalServiceError(format!(
            "Received status code {}",
            meta_response.status()
        )));
    }
    let meta = meta_response
        .json::<FileResponse>()
        .await
        .map_err(|_| StegServiceError::ParsingError)?;
    let is_carrier = meta.is_carrier;
    let filename = meta.filename;

    // Download the actual file bytes
    let mut response = client
        .get(format!("{}/files/{}/download", files_service_url, file_id))
        .bearer_auth(bearer_token)
        .send()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;

    if response.status() != StatusCode::OK {
        return Err(StegServiceError::ExternalServiceError(format!(
            "Received status code {}",
            response.status()
        )));
    }

    let temp_file = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    let mut tokio_temp_file = tokio::fs::File::from_std(
        temp_file
            .as_file()
            .try_clone()
            .map_err(|_| StegServiceError::FileError)?,
    );

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?
    {
        tokio_temp_file
            .write_all(&chunk)
            .await
            .map_err(|_| StegServiceError::FileError)?;
    }
    tokio_temp_file
        .flush()
        .await
        .map_err(|_| StegServiceError::FileError)?;

    let (_file, file_buf) = temp_file.keep().map_err(|_| StegServiceError::FileError)?;

    Ok((file_buf, is_carrier, filename))
}

pub async fn upload_file_to_files_service(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    steg_object_path: PathBuf,
    payload_filename: String,
    carrier_filename: String,
    client: &reqwest::Client,
    files_service_url: &str,
    bearer_token: &str,
) -> Result<FileResponse, StegServiceError> {
    remove_file(&payload_path)
        .await
        .map_err(|_| StegServiceError::FileError)?;
    remove_file(&carrier_path)
        .await
        .map_err(|_| StegServiceError::FileError)?;

    let mut reader = BufReader::new(
        File::open(steg_object_path)
            .await
            .map_err(|_| StegServiceError::FileError)?,
    );
    // S3/MinIO requires each non-final part to be >= 5 MiB.
    // Using a 5 MiB buffer means a small output file becomes a single last-part
    // upload (no minimum size), and a large file gets properly-sized parts.
    let mut buffer = vec![0u8; 5 * 1024 * 1024];
    let mut bytes_read;

    let mut upload_parts: Vec<PartInfo> = vec![];

    let mut response: Response = client
        .post(format!("{}/files/initiate", files_service_url))
        .bearer_auth(bearer_token)
        .send()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;

    if response.status() != StatusCode::CREATED {
        return Err(StegServiceError::ExternalServiceError(format!(
            "Received status code {}",
            response.status()
        )));
    }

    let init_response = response
        .json::<InitiateResponse>()
        .await
        .map_err(|_| StegServiceError::ParsingError)?;

    let mut part_number = 1;
    loop {
        // Fill the buffer completely before uploading. A bare `read()` is not
        // guaranteed to fill the buffer (POSIX contract), so we loop until
        // either the buffer is full or we hit EOF. This ensures every non-final
        // part is exactly 5 MiB — MinIO/S3 rejects parts below that threshold
        // unless they are the last part of the upload.
        bytes_read = 0;
        while bytes_read < buffer.len() {
            let n = reader
                .read(&mut buffer[bytes_read..])
                .await
                .map_err(|_| StegServiceError::FileError)?;
            if n == 0 {
                break;
            }
            bytes_read += n;
        }

        if bytes_read == 0 {
            break;
        }

        response = client
            .post(format!(
                "{}/files/upload-chunk?part_number={}&upload_id={}&object_key={}",
                files_service_url, part_number, init_response.upload_id, init_response.object_key,
            ))
            .bearer_auth(bearer_token)
            .body(buffer[0..bytes_read].to_vec())
            .send()
            .await
            .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;
        if response.status() == StatusCode::OK {
            part_number += 1;
            upload_parts.push(
                response
                    .json::<UploadPartResponse>()
                    .await
                    .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?
                    .part,
            );
        } else {
            return Err(StegServiceError::ExternalServiceError(format!(
                "Received status code {} from files service when trying to upload file part {} of upload id {}",
                response.status().to_string(),
                part_number,
                init_response.upload_id
            )));
        }
    }

    response = client
        .post(format!("{}/files/complete", files_service_url))
        .bearer_auth(bearer_token)
        .json(&CompleteRequest {
            upload_id: init_response.upload_id.clone(),
            object_key: init_response.object_key,
            filename: format!("{} -> {}", payload_filename, carrier_filename),
            parts: upload_parts,
        })
        .send()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;

    if response.status() != StatusCode::CREATED {
        return Err(StegServiceError::ExternalServiceError(format!(
            "Complete request received status code {} expected 201 for upload id: {}",
            response.status(),
            init_response.upload_id
        )));
    }

    let file_response = response
        .json::<FileResponse>()
        .await
        .map_err(|_| StegServiceError::ParsingError)?;

    response = client
        .patch(format!(
            "{}/internal/files/{}/embedded",
            files_service_url, file_response.id
        ))
        .send()
        .await
        .map_err(|e| StegServiceError::ExternalServiceError(e.to_string()))?;

    if response.status() != StatusCode::OK {
        return Err(StegServiceError::ExternalServiceError(format!(
            "Received status code {} from files service when trying to set file {} to be a steg object",
            response.status(),
            file_response.id
        )));
    }

    Ok(file_response)
}
