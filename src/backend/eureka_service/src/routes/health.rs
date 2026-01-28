use axum::Json;

use crate::dtos::HealthResponse;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "eureka_service".to_string(),
    })
}
