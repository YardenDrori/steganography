use axum::extract::State;
use axum::Json;

use crate::app_state::AppState;
use crate::dtos::{RegisterRequest, RegisterResponse};
use crate::errors::eureka_error::EurekaError;
use shared_global::extractors::ValidatedJson;

pub async fn register_service(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<RegisterResponse>, EurekaError> {
    sqlx::query!(
        r#"INSERT INTO service_registry (service_name, service_url, last_heartbeat)
           VALUES ($1, $2, CURRENT_TIMESTAMP)
           ON CONFLICT (service_name)
           DO UPDATE SET service_url = $2, last_heartbeat = CURRENT_TIMESTAMP"#,
        payload.service_name,
        payload.service_url,
    )
    .execute(&state.pool)
    .await?;

    tracing::info!(
        "Service registered: {} -> {}",
        payload.service_name,
        payload.service_url
    );

    Ok(Json(RegisterResponse {
        message: "Service registered successfully".to_string(),
        service_name: payload.service_name,
        service_url: payload.service_url,
    }))
}
