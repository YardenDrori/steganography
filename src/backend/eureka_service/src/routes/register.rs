use crate::app_state::{AppState, ServiceEntry};
use crate::dtos::{RegisterRequest, RegisterResponse};
use axum::extract::State;
use axum::Json;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    // write() locks the HashMap for writing
    let mut services = state.registered_services.write().unwrap();

    let entry = ServiceEntry {
        service_url: payload.service_url.clone(),
        last_heartbeat: tokio::time::Instant::now(),
    };
    services.insert(payload.service_name.clone(), entry);

    tracing::info!(
        "Registered: {} -> {}",
        payload.service_name,
        payload.service_url
    );

    Json(RegisterResponse {
        message: "succesfully registered".to_string(),
        service_url: payload.service_url,
    })
}
