use crate::app_state::AppState;
use crate::dtos::ConfigResponse;
use axum::extract::{Path, State};
use axum::Json;
use base64::{self, engine::general_purpose, Engine};
use std::collections::HashMap;
use std::os::linux::raw::stat;

pub async fn get_config(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> Json<ConfigResponse> {
    let services = state.registered_services.read().unwrap();

    // Only auth_service gets the private key
    // TODO add mTLS validation so that we dont rely just on the service's name
    let jwt_private_key = if service_name == "auth_service" {
        let private_key_string = general_purpose::STANDARD
            .decode(&state.jwt_private_key)
            .expect("Invalid base64 for JWT_PRIVATE_KEY");
        let private_key_string =
            String::from_utf8(private_key_string).expect("JWT_PRIVATE_KEY is not valid UTF-8");
        Some(private_key_string)
    } else {
        None
    };
    let jwt_duration_access_and_refresh: Option<(i64, i64)> = if service_name == "auth_service" {
        Some((
            state.jwt_duration_access_and_refresh.0,
            state.jwt_duration_access_and_refresh.1,
        ))
    } else {
        None
    };

    // Same for public key
    let jwt_public_key = general_purpose::STANDARD
        .decode(&state.jwt_public_key)
        .expect("Invalid base64 for JWT_PUBLIC_KEY");
    let jwt_public_key =
        String::from_utf8(jwt_public_key).expect("JWT_PUBLIC_KEY is not valid UTF-8");

    let services: HashMap<String, String> = services
        .iter()
        .map(|(name, entry)| (name.clone(), entry.service_url.clone()))
        .collect();

    Json(ConfigResponse {
        jwt_public_key,
        jwt_private_key,
        jwt_duration_access_and_refresh,
        services,
    })
}
