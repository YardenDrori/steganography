use crate::errors::steg_service_error::StegServiceError;
use reqwest::StatusCode;
use std::path::PathBuf;
use tempfile;
use tokio::io::AsyncWriteExt;

pub async fn download_file_to_temp(
    client: &reqwest::Client,
    files_service_url: &str,
    file_id: i64,
    bearer_token: &str,
) -> Result<PathBuf, StegServiceError> {
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

    Ok(file_buf)
}
