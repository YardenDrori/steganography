use crate::{
    app_state::AppState,
    dtos::EmbedFileRequest,
    errors::steg_service_error::StegServiceError,
    services::{embed_video::embed, files_client},
};
use axum::{Json, extract::State, http::StatusCode};
use files_dtos::FileResponse;
use shared_global::auth::user_extractors::AuthenticatedUserWithToken;

pub async fn embed_video(
    State(app_state): State<AppState>,
    AuthenticatedUserWithToken(user, access_token): AuthenticatedUserWithToken,
    Json(payload): Json<EmbedFileRequest>,
) -> Result<(StatusCode, Json<FileResponse>), StegServiceError> {
    tracing::info!("User with id: {} attempting to embed video", user);
    let files_service_url = app_state
        .eureka_config
        .read()
        .unwrap()
        .services
        .get("files_service")
        .ok_or(StegServiceError::EurekaConfigError)?
        .to_string();

    let ((carrier_path, is_valid, _, carrier_filename), (payload_path, _, _, payload_filename)) = tokio::try_join!(
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
    if !is_valid {
        tracing::error!("Invalid payload for user: {}", user);
        return Err(StegServiceError::InvalidPayload);
    }
    tracing::info!(
        "Found both carrier and payload files for user: {}. Attmpting to encode video with reed solomon",
        user
    );

    // let payload_path_clone = payload_path.clone();
    // let configs_clone = payload.configs.clone();
    // tokio::task::spawn_blocking(move || {
    //     reed_solomon_encode(payload_path_clone, configs_clone);
    // })
    // .await
    // .map_err(|_| {
    //     StegServiceError::ReedSolomonError("Reed solomon encode task panicked".to_string())
    // })?;

    //since steg work is CPU bound thus blocking we make a dedicated thread for it to not starve
    //other async processes in this step
    let payload_path_clone = payload_path.clone();
    let carrier_path_clone = carrier_path.clone();
    let output_path = tokio::task::spawn_blocking(move || {
        embed(payload_path_clone, carrier_path_clone, payload.configs)
    })
    .await
    .map_err(|_| StegServiceError::Other("embed task panicked".to_string()))??;

    tracing::info!("Successfully embedded video. Attemoting to upload to files service");

    let steg_file_remote_pointer = files_client::upload_file_to_files_service(
        payload_path,
        carrier_path,
        output_path,
        payload_filename,
        carrier_filename,
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

    Ok((StatusCode::CREATED, Json(steg_file_remote_pointer)))
}
