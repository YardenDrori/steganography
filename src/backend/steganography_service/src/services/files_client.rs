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
) -> Result<(PathBuf, bool), StegServiceError> {
    let mut response = client
        .get(format!("{}/{}", files_service_url, file_id))
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

    response = client
        .get(format!("{}/{}", files_service_url, file_id))
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
    let is_carrier = response
        .json::<FileResponse>()
        .await
        .map_err(|_| StegServiceError::ParsingError)?
        .is_carrier;

    Ok((file_buf, is_carrier))
}

pub async fn upload_file_to_files_service(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    steg_object_path: PathBuf,
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
    let mut buffer = [0u8; 4096];
    let mut bytes_read;

    let mut upload_parts: Vec<PartInfo> = vec![];

    let mut response: Response = client
        .post(format!("{}/initiate", files_service_url))
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
        bytes_read = reader
            .read(&mut buffer)
            .await
            .map_err(|_| StegServiceError::FileError)?;

        if bytes_read == 0 {
            break;
        }

        response = client
            .post(format!(
                "{}/upload-chunk?part_number={}&upload_id={}&object_key={}",
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

    let carrier_filename = carrier_path
        .file_name()
        .ok_or(StegServiceError::FileError)?;
    let payload_filename = payload_path
        .file_name()
        .ok_or(StegServiceError::FileError)?;
    response = client
        .post(format!("{}/complete", files_service_url))
        .bearer_auth(bearer_token)
        .json(&CompleteRequest {
            upload_id: init_response.upload_id.clone(),
            object_key: init_response.object_key,
            filename: format!(
                "{} -> {}",
                payload_filename.to_string_lossy(),
                carrier_filename.to_string_lossy()
            ),
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

    Ok(file_response)
}
