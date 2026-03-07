use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use shared_global::auth::user_extractors::AuthenticatedUserIsAdmin;

use crate::{
    app_state::AppState,
    dtos::RenameFileRequest,
    errors::files_service_errors::FilesServiceError,
    services::{self},
};

pub async fn rename_file(
    State(app_state): State<AppState>,
    AuthenticatedUserIsAdmin(user, is_admin): AuthenticatedUserIsAdmin,
    Path(file_id): Path<i64>,
    Json(payload): Json<RenameFileRequest>,
) -> Result<StatusCode, FilesServiceError> {
    tracing::info!(
        "User with id {} requested to rename file with object_id {}",
        user,
        file_id
    );
    services::files_service::rename_file(
        &app_state.pool,
        file_id,
        user,
        is_admin,
        &payload.new_name,
    )
    .await
    .map_err(|e| {
        tracing::info!(
            "Failed to rename file with id {} belonging to user {}. error: {:?}",
            file_id,
            user,
            e
        );
        e
    })?;
    tracing::info!(
        "Renamed file with id {} belonging to user {} to {}",
        file_id,
        user,
        payload.new_name
    );
    Ok(StatusCode::OK)
}

pub async fn set_file_embdedded(
    State(app_state): State<AppState>,
    Path(file_id): Path<i64>,
) -> Result<StatusCode, FilesServiceError> {
    tracing::info!("setting file with id {} to be a steg object", file_id);
    services::files_service::set_is_steg_object(&app_state.pool, file_id, true)
        .await
        .map_err(|e| {
            tracing::info!(
                "Failed to set file with id {} to be a steg object. error: {:?}",
                file_id,
                e
            );
            e
        })?;
    tracing::info!("set file with id {} to be a steg object", file_id);
    Ok(StatusCode::OK)
}
