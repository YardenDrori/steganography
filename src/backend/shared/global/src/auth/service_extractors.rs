use crate::errors::ErrorBody;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};

//TODO: add mTLS verification currently this does nothing
pub struct InternalService;

#[async_trait]
impl<S> FromRequestParts<S> for InternalService
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        tracing::warn!("Called unimplemented method: InternalService: FromRequestParts");
        Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(ErrorBody::new("Used unimplemented method!")),
        ))
    }
}
