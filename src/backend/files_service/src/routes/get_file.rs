use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::app_state::AppState;
use crate::dtos::FileResponse;
use crate::errors::file_service_error::FileServiceError;

const DOWNLOAD_URL_EXPIRY_SECS: u64 = 3600; // 1 hour

pub async fn get_file(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(file_id): Path<i64>,
) -> Result<(StatusCode, Json<FileResponse>), FileServiceError> {
    let file = sqlx::query!(
        r#"
        SELECT id, user_id, filename, content_type, size_bytes, status, created_at, minio_object_key
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

    let download_url = if file.status == "uploaded" {
        let presigned = state
            .minio
            .get_object_presigned(&state.minio_bucket, &file.minio_object_key, DOWNLOAD_URL_EXPIRY_SECS)
            .await
            .map_err(|e| FileServiceError::MinioError(e.to_string()))?;

        presigned.map(|p| p.uri().to_string())
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(FileResponse {
            id: file.id,
            filename: file.filename,
            content_type: file.content_type,
            size_bytes: file.size_bytes,
            status: file.status,
            created_at: file.created_at,
            download_url,
        }),
    ))
}
