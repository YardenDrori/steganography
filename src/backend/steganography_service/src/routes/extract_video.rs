use crate::{
    app_state::AppState,
    dtos::ExtractFileRequest,
    errors::steg_service_error::StegServiceError,
    services::{extract_file::extract, files_client, reed_solomon::reed_solomon_decode},
};
use axum::{Json, extract::State, http::StatusCode};
use files_dtos::FileResponse;
use shared_global::auth::user_extractors::AuthenticatedUserWithToken;

pub async fn extract_video(
    State(app_state): State<AppState>,
    AuthenticatedUserWithToken(user, access_token): AuthenticatedUserWithToken,
    Json(payload): Json<ExtractFileRequest>,
) -> Result<(StatusCode, Json<FileResponse>), StegServiceError> {
    tracing::info!("User with id: {} attempting to extract video", user);
    let files_service_url = app_state
        .eureka_config
        .read()
        .unwrap()
        .services
        .get("files_service")
        .ok_or(StegServiceError::EurekaConfigError)?
        .to_string();

    let (steg_object_path, _, is_steg_object, filename) = files_client::download_file_to_temp(
        &app_state.client,
        &files_service_url,
        payload.steg_object_id,
        &access_token,
    )
    .await?;

    if !is_steg_object {
        tracing::error!(
            "File {} is not a steg object, user: {}",
            payload.steg_object_id,
            user
        );
        return Err(StegServiceError::InvalidPayload);
    }
    tracing::info!(
        "Found steg object for user: {}. Attempting to extract payload",
        user
    );

    let steg_object_path_clone = steg_object_path.clone();
    let configs_clone = payload.configs.clone();
    let output_path =
        tokio::task::spawn_blocking(move || extract(steg_object_path_clone, configs_clone))
            .await
            .map_err(|_| StegServiceError::Other("extract task panicked".to_string()))??;

    tracing::info!("Extraction complete. Applying Reed-Solomon decode to extracted payload");

    let output_path_clone = output_path.clone();
    let configs_for_rs = payload.configs.clone();
    tokio::task::spawn_blocking(move || reed_solomon_decode(output_path_clone, &configs_for_rs))
        .await
        .map_err(|_| {
            StegServiceError::ReedSolomonError("Reed solomon decode task panicked".to_string())
        })??;

    tracing::info!("Successfully extracted and decoded payload. Attempting to upload to files service");

    let mut payload_filename = filename
        .split(" -> ")
        .next()
        .unwrap_or(&filename)
        .to_string();
    payload_filename = format!("{}(1)", payload_filename);

    let extracted_file = files_client::upload_extracted_file(
        steg_object_path,
        output_path,
        payload_filename,
        &app_state.client,
        &files_service_url,
        &access_token,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to upload extracted file to files service");
        e
    })?;

    tracing::info!(
        "Successfully uploaded extracted file. Extraction pipeline complete for user: {}",
        user
    );

    Ok((StatusCode::CREATED, Json(extracted_file)))
}
