use std::path::PathBuf;

use crate::{
    app_state::AppState, dtos::EmbedFileRequest, errors::steg_service_error::StegServiceError,
    services::files_client,
};
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use shared_global::auth::user_extractors::AuthenticatedUserWithToken;

pub async fn embed_video(
    State(app_state): State<AppState>,
    AuthenticatedUserWithToken(user, access_token): AuthenticatedUserWithToken,
    Json(payload): Json<EmbedFileRequest>,
) -> Result<(StatusCode, Json<i64>), StegServiceError> {
    let files_service_url = app_state
        .eureka_config
        .read()
        .unwrap()
        .services
        .get("files_service")
        .ok_or(StegServiceError::EurekaConfigError)?
        .to_string();

    let (carrier_path, payload_path) = tokio::try_join!(
        files_client::download_file_to_temp(
            &app_state.client,
            &files_service_url,
            payload.carrier_id,
            &access_token,
        ),
        files_client::download_file_to_temp(
            &app_state.client,
            &files_service_url,
            payload.payload_id,
            &access_token,
        )
    )?;

    todo!()
}
