use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::app_state::AppState;
use crate::dtos::{FileListResponse, FileResponse, ListFilesQuery};
use crate::errors::file_service_error::FileServiceError;

pub async fn list_files(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<ListFilesQuery>,
) -> Result<(StatusCode, Json<FileListResponse>), FileServiceError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!: i64"
        FROM files
        WHERE user_id = $1 AND deleted_at IS NULL
        "#,
        user_id,
    )
    .fetch_one(&state.pool)
    .await?;

    let rows = sqlx::query!(
        r#"
        SELECT id, filename, content_type, size_bytes, status, created_at
        FROM files
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        per_page,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    let files = rows
        .into_iter()
        .map(|r| FileResponse {
            id: r.id,
            filename: r.filename,
            content_type: r.content_type,
            size_bytes: r.size_bytes,
            status: r.status,
            created_at: r.created_at,
            download_url: None,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(FileListResponse {
            files,
            total,
            page,
            per_page,
        }),
    ))
}
