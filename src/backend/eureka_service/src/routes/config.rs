use axum::extract::{Path, State};
use axum::Json;
use std::collections::HashMap;

use crate::app_state::AppState;
use crate::dtos::ConfigResponse;
use crate::errors::eureka_error::EurekaError;

pub async fn get_config(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> Result<Json<ConfigResponse>, EurekaError> {
    // Load registered services from DB
    let rows = sqlx::query!(
        "SELECT service_name, service_url FROM service_registry"
    )
    .fetch_all(&state.pool)
    .await?;

    let mut services: HashMap<String, String> = rows
        .into_iter()
        .map(|r| (r.service_name, r.service_url))
        .collect();

    // Fallback defaults for known services (in case they haven't registered yet)
    // SERVICE_HOST controls whether we use Docker hostnames or localhost
    let host = &state.service_host;
    let defaults = [
        ("auth_service", format!("http://{}:3001", if host == "localhost" { "localhost" } else { "auth_service" })),
        ("user_service", format!("http://{}:3002", if host == "localhost" { "localhost" } else { "user_service" })),
        ("steganography_service", format!("http://{}:3003", if host == "localhost" { "localhost" } else { "steganography_service" })),
        ("files_service", format!("http://{}:3004", if host == "localhost" { "localhost" } else { "files_service" })),
    ];
    for (name, url) in defaults {
        services
            .entry(name.to_string())
            .or_insert_with(|| url);
    }

    // Only auth_service gets the private key
    let jwt_private_key = if service_name == "auth_service" {
        Some(state.jwt_private_key.clone())
    } else {
        None
    };

    Ok(Json(ConfigResponse {
        jwt_public_key: state.jwt_public_key.clone(),
        jwt_private_key,
        internal_api_key: state.internal_api_key.clone(),
        services,
    }))
}
