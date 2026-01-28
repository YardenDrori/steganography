use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::app_state::AppState;
use crate::dtos::ConfirmUploadResponse;
use crate::errors::file_service_error::FileServiceError;

pub async fn confirm_upload(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(file_id): Path<i64>,
) -> Result<(StatusCode, Json<ConfirmUploadResponse>), FileServiceError> {
    let file = sqlx::query!(
        r#"
        SELECT id, user_id, filename, minio_object_key, status
        FROM files
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        file_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(FileServiceError::NotFound)?;

    if file.user_id != user_id {
        return Err(FileServiceError::Unauthorized);
    }

    if file.status == "uploaded" {
        return Err(FileServiceError::FileAlreadyConfirmed);
    }

    // Verify the object actually exists in MinIO
    let exists = state
        .minio
        .object_exists(&state.minio_bucket, &file.minio_object_key)
        .await
        .map_err(|e| FileServiceError::MinioError(e.to_string()))?;

    if !exists {
        return Err(FileServiceError::FileNotReady);
    }

    // Get object metadata to record size
    let objects = state
        .minio
        .list_bucket_objects(&state.minio_bucket)
        .await
        .map_err(|e| FileServiceError::MinioError(e.to_string()))?;

    let size_bytes = objects
        .iter()
        .find(|o| o.key().is_some_and(|k| k == file.minio_object_key))
        .and_then(|o| o.size())
        .unwrap_or(0);

    sqlx::query!(
        r#"
        UPDATE files
        SET status = 'uploaded', size_bytes = $1
        WHERE id = $2
        "#,
        size_bytes,
        file_id,
    )
    .execute(&state.pool)
    .await?;

    tracing::info!(
        "User {} confirmed upload for file '{}' (id={}, size={})",
        user_id,
        file.filename,
        file_id,
        size_bytes
    );

    Ok((
        StatusCode::OK,
        Json(ConfirmUploadResponse {
            id: file_id,
            filename: file.filename,
            size_bytes,
            status: "uploaded".to_string(),
        }),
    ))
}
