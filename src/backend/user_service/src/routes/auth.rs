use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use crate::{app_state::AppState, dtos::VerifyCredentialsRequest, services::user_service};
use shared_global::auth::internal_auth::AdminOrInternal;

pub async fn verify_credentials(
    State(app_state): State<AppState>,
    AdminOrInternal: AdminOrInternal,
    Json(payload): Json<VerifyCredentialsRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user = user_service::verify_credentials(&app_state.pool, &payload).await?;
    Ok((StatusCode::OK, Json(user)))
}
