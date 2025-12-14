use crate::app_state::AppState;
use crate::services::embed_image;
use axum::{extract::State, http::StatusCode};
use shared_global::auth::service_extractors::InternalService;

pub async fn embed_image(
    InternalService: InternalService,
    State(_app_state): State<AppState>,
    /*todo return error here*/
) -> Result<StatusCode, ()> {
    embed_image::embed_image().await;
    Ok(StatusCode::OK)
}
