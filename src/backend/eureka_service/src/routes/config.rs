use crate::app_state::AppState;
use crate::dtos::ConfigResponse;
use axum::extract::{Path, State};
use axum::Json;
use std::collections::HashMap;

pub async fn get_config(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> Json<ConfigResponse> {
    let services = state.registered_services.read().unwrap();

    // Only auth_service gets the private key
    // TODO add mTLS validation so that we dont rely just on the service's name
    let jwt_private_key = if service_name == "auth_service" {
        Some(state.jwt_private_key.clone())
    } else {
        None
    };

    let services: HashMap<String, String> = services
        .iter()
        .map(|(name, entry)| (name.clone(), entry.service_url.clone()))
        .collect();

    Json(ConfigResponse {
        jwt_public_key: state.jwt_public_key.clone(),
        jwt_private_key,
        services: services,
    })
}
