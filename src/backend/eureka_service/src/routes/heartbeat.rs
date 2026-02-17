use crate::app_state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use shared_global::dtos::EurekaHeartBeatRequest;
use tokio::time::Instant;

pub async fn heartbeat(
    State(state): State<AppState>,
    Json(payload): Json<EurekaHeartBeatRequest>,
) -> StatusCode {
    let mut services = state.registered_services.write().unwrap();

    tracing::info!("Received heartbeat request from {}.", payload.service_name);

    if !services.contains_key(&payload.service_name) {
        tracing::warn!(
            "Could not find matching key pair for {} in registered_services.",
            payload.service_name
        );
        return StatusCode::NOT_FOUND;
    }
    services
        .get_mut(&payload.service_name)
        .unwrap()
        .last_heartbeat = Instant::now();

    StatusCode::OK
}
