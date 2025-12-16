use crate::app_state::AppState;
use crate::dtos::PrepareResponse;
use crate::errors::file_service_error::FileServiceError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use shared_global::auth::user_extractors::AuthenticatedUser;

pub async fn post_files(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<(StatusCode, Json<PrepareResponse>), (StatusCode, FileServiceError)> {
    tracing::info!("User {} attempting to upload file", user_id);
    todo!();
}
