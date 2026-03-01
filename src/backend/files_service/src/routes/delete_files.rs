use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use shared_global::auth::user_extractors::AuthenticatedUserIsAdmin;

use crate::{
    app_state::AppState,
    errors::files_service_errors::FilesServiceError,
    services::{self},
};

pub async fn delete_file(
    State(app_state): State<AppState>,
    AuthenticatedUserIsAdmin(user, is_admin): AuthenticatedUserIsAdmin,
    Path(file_id): Path<i64>,
) -> Result<StatusCode, FilesServiceError> {
    tracing::info!(
        "User with id {} requested deletion of file with object_id {}",
        user,
        file_id
    );
    services::files_service::delete_file(
        &app_state.bucket,
        &app_state.pool,
        file_id,
        user,
        is_admin,
    )
    .await
    .map_err(|e| {
        tracing::info!(
            "Failed to delete file with object_id {} owned by user with id {}. error: {:?}",
            file_id,
            user,
            e
        );
        e
    })?;
    tracing::info!(
        "Deleted file with object_id {} owned by user with id {}",
        file_id,
        user
    );
    Ok(StatusCode::NO_CONTENT)
}
