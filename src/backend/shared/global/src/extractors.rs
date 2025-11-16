use crate::errors::ErrorBody;
use async_trait::async_trait;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::Json;
use serde::de::DeserializeOwned;
use validator::Validate;

/// JSON extractor that automatically validates the payload using the `validator` crate.
///
/// This extractor deserializes JSON and runs validation, returning appropriate errors
/// if either step fails. Use this instead of `Json<T>` when you want automatic validation.
///
/// # Example
/// ```rust,ignore
/// async fn create_user(
///     ValidatedJson(payload): ValidatedJson<UserCreateRequest>,
/// ) -> Result<...> {
///     // payload is already validated
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // First, deserialize the JSON
        let Json(payload) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_err| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorBody::new("Invalid JSON format")),
                )
            })?;

        // Then validate the payload
        payload.validate().map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new(&format!("Validation error: {:?}", e))),
            )
        })?;

        Ok(ValidatedJson(payload))
    }
}
