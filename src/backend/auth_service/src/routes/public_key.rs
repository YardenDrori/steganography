use axum::{extract::State, http::StatusCode, Json};
use crate::app_state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct PublicKeyResponse {
    pub public_key: String,
}

/// Public endpoint that returns the JWT public key for token verification
/// This allows other services to fetch the public key without storing it
pub async fn get_public_key(
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<PublicKeyResponse>), StatusCode> {
    Ok((
        StatusCode::OK,
        Json(PublicKeyResponse {
            public_key: app_state.jwt_public_key.clone(),
        }),
    ))
}
