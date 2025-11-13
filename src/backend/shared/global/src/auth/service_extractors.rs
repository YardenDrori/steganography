use crate::auth::internal::HasInternalApiKey;
use crate::errors::ErrorBody;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};

pub struct InternalService;

#[async_trait]
impl<S> FromRequestParts<S> for InternalService
where
    S: Send + Sync + HasInternalApiKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let expected_key = state.internal_api_key();

        let header = parts
            .headers
            .get("X-Internal-API-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            ))?;

        if header == expected_key {
            Ok(InternalService)
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            ))
        }
    }
}
