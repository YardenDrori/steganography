use crate::auth::internal::HasInternalApiKey;
use crate::auth::jwt::{verify_jwt, HasJwtSecret};
use crate::auth::roles::Role;
use crate::errors::ErrorBody;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};

/// Extractor that allows either:
/// 1. Admin users (via JWT with admin role)
/// 2. Internal services (via X-Internal-API-Key header)
///
/// Returns None if internal service, Some(user_id) if admin user
pub struct AdminOrInternal(pub Option<i64>);

#[async_trait]
impl<S> FromRequestParts<S> for AdminOrInternal
where
    S: Send + Sync + HasJwtSecret + HasInternalApiKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        //try internal service auth (check for X-Internal-API-Key header)
        let internal_api_key = state.internal_api_key();
        if let Some(header) = parts
            .headers
            .get("X-Internal-API-Key")
            .and_then(|h| h.to_str().ok())
        {
            if header == internal_api_key {
                return Ok(AdminOrInternal(None));
            }
        }

        // If internal auth failed try JWT admin auth
        let jwt_secret = state.jwt_secret();
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            ))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .or_else(|| auth_header.strip_prefix("bearer "))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            ))?;

        let claims = verify_jwt(&token, &jwt_secret).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            )
        })?;

        // Check if user has admin role
        for role in claims.roles {
            if role == Role::Admin {
                return Ok(AdminOrInternal(Some(claims.sub))); // Admin user
            }
        }

        // Not internal service and not admin
        Err((StatusCode::FORBIDDEN, Json(ErrorBody::new("Forbidden"))))
    }
}
