use axum::Json;
use shared_global::dtos::HealthResponse;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
