use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::{
    app_state::AppState,
    dtos::{CompleteRequest, FileResponse, InitiateResponse, UploadPartResponse},
    errors::files_service_errors::FilesServiceError,
    services::{
        self,
        files_service::{self, initiate_upload},
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadPartQuery {
    part_number: u32,
    upload_id: String,
    object_key: String,
}

pub async fn initiate(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<(StatusCode, Json<InitiateResponse>), FilesServiceError> {
    tracing::info!("User with id: {}, attempting to initiate file upload", user);
    let response = initiate_upload(&app_state.bucket).await.map_err(|e| {
        tracing::info!(
            "user with id: {}, failed to initiate upload error: {:?}",
            user,
            e
        );
        e
    })?;
    tracing::info!("User with id: {} succesfully initiated upload", user);
    Ok((StatusCode::CREATED, Json(response)))
}
pub async fn upload_chunk(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<UploadPartQuery>,
    chunk: Bytes,
) -> Result<(StatusCode, Json<UploadPartResponse>), FilesServiceError> {
    tracing::trace!(
        "User with id: {}, uploaded chunk with object_key: {}, part_number: {}, upload_id: {}",
        user,
        &query.object_key,
        &query.part_number,
        &query.upload_id
    );
    let response = files_service::upload_chunk(
        &app_state.bucket,
        chunk.to_vec(),
        &query.object_key,
        query.part_number,
        &query.upload_id,
    )
    .await.map_err(|e| {
        tracing::info!("failed to process chunk (object_key: {}, part_number: {}, upload_id: {}) from user withj id: {}. error: {:?}", query.object_key, query.part_number, query.upload_id, user, e);
        e
    })?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn complete_upload(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CompleteRequest>,
) -> Result<(StatusCode, Json<FileResponse>), FilesServiceError> {
    tracing::info!(
        "received complete upload request from user with id {}",
        user
    );
    let response = services::files_service::complete_upload(
        &app_state.bucket,
        &app_state.pool,
        payload.clone(),
        user,
    )
    .await
    .map_err(|e| {
        tracing::info!(
            "Faild to complete upload for upload_id: {} for user with id {}. error: {:?}",
            payload.upload_id,
            user,
            e
        );
        e
    })?;

    tracing::info!(
        "succesfully completed uploading file from user with id {}",
        user
    );
    Ok((StatusCode::CREATED, Json(response)))
}
