use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::app_state::AppState;
use crate::dtos::DeleteFileResponse;
use crate::errors::file_service_error::FileServiceError;

pub async fn delete_file(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(file_id): Path<i64>,
) -> Result<(StatusCode, Json<DeleteFileResponse>), FileServiceError> {
    let file = sqlx::query!(
        r#"
        SELECT id, user_id, minio_object_key
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

    // Delete from MinIO
    state
        .minio
        .delete_object(&state.minio_bucket, &file.minio_object_key)
        .await
        .map_err(|e| FileServiceError::MinioError(e.to_string()))?;

    // Soft delete in database
    sqlx::query!(
        r#"
        UPDATE files
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        file_id,
    )
    .execute(&state.pool)
    .await?;

    tracing::info!("User {} deleted file id={}", user_id, file_id);

    Ok((
        StatusCode::OK,
        Json(DeleteFileResponse {
            message: "File deleted successfully".to_string(),
        }),
    ))
}
