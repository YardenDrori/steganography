use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_global::auth::user_extractors::AuthenticatedUser;
use shared_global::extractors::ValidatedJson;

use crate::{
    app_state::AppState, dtos::InitiateResponse, errors::files_service_errors::FilesServiceError,
    services::files_service::initiate_upload,
};

pub async fn initiate(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<(StatusCode, Json<InitiateResponse>), FilesServiceError> {
    tracing::info!("User with id: {}, attempting to initiate file upload", user);
    let response = initiate_upload(&app_state.bucket).await.map_err(|e| {
        tracing::info!(
            "user with id: {}, failed to initiate upload error: {:?}",
            user,
            e
        );
        e
    })?;
    tracing::info!("User with id: {} succesfully initiated upload", user);
    Ok((StatusCode::CREATED, Json(response)))
}
