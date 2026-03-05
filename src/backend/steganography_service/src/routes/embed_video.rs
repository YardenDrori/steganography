use crate::{
    app_state::AppState,
    dtos::EmbedFileRequest,
    errors::steg_service_error::StegServiceError,
    services::{embed_video::embed, files_client},
};
use axum::{Json, extract::State, http::StatusCode};
use shared_global::auth::user_extractors::AuthenticatedUserWithToken;

pub async fn embed_video(
    State(app_state): State<AppState>,
    AuthenticatedUserWithToken(user, access_token): AuthenticatedUserWithToken,
    Json(payload): Json<EmbedFileRequest>,
) -> Result<(StatusCode, Json<i64>), StegServiceError> {
    tracing::info!("User with id: {} attempting to embed video", user);
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
    tracing::info!("Found both carrier and payload files for user: {}", user);

    let output_path =
        embed(payload_path.clone(), carrier_path.clone(), payload.configs).map_err(|e| {
            tracing::error!("Failed to embed video");
            e
        })?;
    tracing::info!("Successfully embedded video. Attemoting to upload to files service");

    files_client::upload_file_to_files_service(
        payload_path,
        carrier_path,
        output_path,
        &app_state.client,
        &files_service_url,
        &access_token,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to upload file to files service");
        e
    })?;

    tracing::info!(
        "Succesfully uploaded file to files service. Steganography pipeline complete for user: {}",
        user
    );

    Ok((StatusCode::OK, Json(payload.payload_id)))
}
