use crate::{
    app_state::AppState, dtos::VerifyCredentialsRequest,
    errors::user_service_errors::UserServiceError, services::user_service,
};
use axum::{extract::State, http::StatusCode, Json};
use shared_global::auth::hybrid_extractors::AdminOrInternal;

pub async fn verify_credentials(
    State(app_state): State<AppState>,
    _auth: AdminOrInternal,
    Json(payload): Json<VerifyCredentialsRequest>,
) -> Result<(StatusCode, Json<crate::dtos::UserResponse>), UserServiceError> {
    let user = user_service::verify_credentials(&app_state.pool, &payload).await?;
    Ok((StatusCode::OK, Json(user)))
}
