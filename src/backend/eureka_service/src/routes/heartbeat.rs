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
    if doheartbeat(&state, &payload.service_name).await {
        return StatusCode::OK;
    }
    return StatusCode::NOT_FOUND;
}
pub async fn doheartbeat(state: &AppState, service_name: &str) -> bool {
    let mut services = state.registered_services.write().unwrap();

    tracing::info!("Received heartbeat request from {}.", &service_name);

    if !services.contains_key(service_name) {
        tracing::warn!(
            "Could not find matching key pair for {} in registered_services.",
            &service_name
        );
        return false;
    }
    services.get_mut(service_name).unwrap().last_heartbeat = Instant::now();

    return true;
}
