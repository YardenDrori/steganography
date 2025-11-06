use crate::app_state;
use crate::auth::jwt::verify_jwt;
use axum::Json;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use shared::errors::ErrorBody;

//we use a user id wrapper to implement for it the trait FromRequestParts that way whenever the
//struct is passed as a handler input it will authenticate the user automatically
pub struct AuthUser(pub i64);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: app_state::HasJwtSecret + Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_secret = state.jwt_secret();

        //get headers from request parts
        let headers = &parts.headers;
        //get the authorization header
        let auth_header = headers.get("authorization").ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody::new("No authrization header in request headers")),
        ))?;
        //stringify the header
        let auth_header = auth_header
            .to_str()
            .map_err(|_| {
                tracing::error!("Failed to convert auth header to string");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorBody::new("Invalid Authorization header format")),
                )
            })?
            .to_string();
        // strip bearer prefix to get only the token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Authorization must start with \"Bearer \"")),
            ))?
            .to_string();

        let claims = verify_jwt(&token, &jwt_secret).map_err(|e| {
            tracing::error!("Jwt error: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Invalid or expired token")),
            )
        })?;
        let user_id = claims.sub;

        Ok(AuthUser(user_id))
    }
}
