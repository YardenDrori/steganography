use crate::app_state::AppState;
use crate::dtos::{RegisterRequest, RegisterResponse};
use axum::extract::State;
use axum::Json;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    // write() locks the HashMap for writing
    let mut services = state.registered_services.write().unwrap();
    services.insert(payload.service_name.clone(), payload.service_url.clone());

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
