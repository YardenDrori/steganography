use axum::extract::{Path, State};
use axum::Json;

use crate::app_state::AppState;
use crate::dtos::DiscoverResponse;
use crate::errors::eureka_error::EurekaError;

pub async fn discover_service(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> Result<Json<DiscoverResponse>, EurekaError> {
    let row = sqlx::query!(
        "SELECT service_name, service_url FROM service_registry WHERE service_name = $1",
        service_name,
    )
    .fetch_optional(&state.pool)
    .await?;

    match row {
        Some(r) => Ok(Json(DiscoverResponse {
            service_name: r.service_name,
            service_url: r.service_url,
        })),
        None => Err(EurekaError::NotFound(service_name)),
    }
}
