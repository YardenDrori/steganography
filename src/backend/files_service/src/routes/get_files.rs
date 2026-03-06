use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    Json,
};
use shared_global::auth::user_extractors::{AuthenticatedUser, AuthenticatedUserIsAdmin};

use crate::{
    app_state::AppState,
    dtos::FileResponse,
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

pub async fn get_file_by_id(
    State(app_state): State<AppState>,
    AuthenticatedUserIsAdmin(user, is_admin): AuthenticatedUserIsAdmin,
    Path(file_id): Path<i64>,
) -> Result<(StatusCode, Json<FileResponse>), FilesServiceError> {
    tracing::info!(
        "User with id {} requested to view file with id {}",
        user,
        file_id
    );
    let file = services::files_service::find_file_by_id(&app_state.pool, file_id, user, is_admin)
        .await
        .map_err(|e| {
            tracing::info!(
                "Failed to retrieve file with id {} for user with id {}, error: {:?}",
                file_id,
                user,
                e
            );
            e
        })?
        .ok_or(FilesServiceError::NotFound)?;
    tracing::info!(
        "Retrieved file with id {} for user with id {} successfully",
        file_id,
        user
    );
    Ok((StatusCode::OK, Json(file)))
}

pub async fn download_file(
    State(app_state): State<AppState>,
    AuthenticatedUserIsAdmin(user, is_admin): AuthenticatedUserIsAdmin,
    Path(file_id): Path<i64>,
) -> Result<Response, FilesServiceError> {
    tracing::info!(
        "User with id {} requested download of file with id {}",
        user,
        file_id
    );
    let stream = services::files_service::download_file(
        &app_state.bucket,
        &app_state.pool,
        user,
        is_admin,
        file_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to stream file {} to user {}. error: {:?}",
            file_id,
            user,
            e
        );
        e
    })?;
    tracing::info!("Streaming file {} to user {}", file_id, user);
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(stream.bytes))
        .unwrap();
    Ok(response)
}
