use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;
use shared_global::extractors::ValidatedJson;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::dtos::{PrepareUploadRequest, PrepareUploadResponse};
use crate::errors::file_service_error::FileServiceError;

const PRESIGNED_URL_EXPIRY_SECS: u64 = 3600; // 1 hour

pub async fn prepare_upload(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<PrepareUploadRequest>,
) -> Result<(StatusCode, Json<PrepareUploadResponse>), FileServiceError> {
    let object_key = format!("{}/{}/{}", user_id, Uuid::new_v4(), payload.filename);

    let record = sqlx::query_scalar!(
        r#"
        INSERT INTO files (user_id, filename, minio_object_key, content_type, status)
        VALUES ($1, $2, $3, $4, 'pending')
        RETURNING id
        "#,
        user_id,
        payload.filename,
        object_key,
        payload.content_type,
    )
    .fetch_one(&state.pool)
    .await?;

    let presigned = state
        .minio
        .upload_object_presigned(&state.minio_bucket, &object_key, PRESIGNED_URL_EXPIRY_SECS)
        .await
        .map_err(|e| FileServiceError::MinioError(e.to_string()))?;

    tracing::info!(
        "User {} prepared upload for file '{}' (id={}, key={})",
        user_id,
        payload.filename,
        record,
        object_key
    );

    Ok((
        StatusCode::CREATED,
        Json(PrepareUploadResponse {
            file_id: record,
            upload_url: presigned.uri().to_string(),
            expires_in: PRESIGNED_URL_EXPIRY_SECS,
        }),
    ))
}
