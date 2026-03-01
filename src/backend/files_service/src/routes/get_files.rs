use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use shared_global::auth::user_extractors::{AuthenticatedUser, AuthenticatedUserIsAdmin};

use crate::{
    app_state::AppState,
    dtos::{DownloadResponse, FileResponse},
    errors::files_service_errors::FilesServiceError,
    services::{self, files_service::list_files},
};

pub async fn get_files_for_user(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<(StatusCode, Json<Vec<FileResponse>>), FilesServiceError> {
    tracing::info!("User with id {} requested own file list", user);
    let files_found = list_files(&app_state.pool, user).await.map_err(|e| {
        tracing::info!(
            "Failed to retrieve file list for user with id {}, error: {:?}",
            user,
            e
        );
        e
    })?;
    tracing::info!(
        "Retrieved file list for user with id {}  successfully",
        user
    );
    Ok((StatusCode::OK, Json(files_found)))
}

pub async fn get_download_url(
    State(app_state): State<AppState>,
    AuthenticatedUserIsAdmin(user, is_admin): AuthenticatedUserIsAdmin,
    Path(file_id): Path<i64>,
) -> Result<(StatusCode, Json<DownloadResponse>), FilesServiceError> {
    tracing::info!(
        "User with id {} requested download url for file with object_id {}",
        user,
        file_id
    );
    let url = services::files_service::get_download_url(
        &app_state.bucket,
        &app_state.pool,
        user,
        is_admin,
        file_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to send download link back to user with id {}. error: {:?}",
            user,
            e
        );
        e
    })?;
    tracing::info!(
        "successfully sent download url to user with id {} for file with object_id {}",
        user,
        file_id
    );
    Ok((StatusCode::OK, Json(url)))
}
