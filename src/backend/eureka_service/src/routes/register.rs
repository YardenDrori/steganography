use crate::app_state::{AppState, ServiceEntry};
use crate::dtos::{RegisterRequest, RegisterResponse};
use crate::routes::heartbeat::doheartbeat;
use axum::extract::State;
use axum::Json;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    // write() locks the HashMap for writing
    // potetntially race condition??? idk how to fix this tho
    let services = state.registered_services.read().unwrap().clone();

    if services.contains_key(&payload.service_name) {
        tracing::warn!(
            "Service {} attempted to register with eureka but was already registered, attempting to do heartbeat instead",
            &payload.service_name
        );
        if doheartbeat(&state, &payload.service_name).await {
            return Json(RegisterResponse {
                message: "send register request while already registered interperted as heartbeat."
                    .to_string(),
                service_url: payload.service_url.clone(),
            });
        } else {
            return Json(RegisterResponse {
                message: "send register request while already registered failed to interpret as heartbeat."
                    .to_string(),
                service_url: payload.service_url.clone(),
            });
        }
    }
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
