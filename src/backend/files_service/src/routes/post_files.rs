use axum::extract::State;
use axum::http::StatusCode;
use shared_global::auth::user_extractors::AuthenticatedUser;

use crate::app_state::AppState;

pub async fn post_files(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<StatusCode, ()> {
    tracing::info!("User {} attempting to upload file", user_id);
    todo!();
}
