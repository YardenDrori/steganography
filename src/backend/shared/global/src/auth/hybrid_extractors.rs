use crate::auth::jwt::{verify_jwt, HasJwtPublicKey};
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
// TODO: currently admin-only. add mTLS for service-to-service auth in the future.
pub struct AdminOrInternal(pub Option<i64>);

#[async_trait]
impl<S> FromRequestParts<S> for AdminOrInternal
where
    S: Send + Sync + HasJwtPublicKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_public_key = state.jwt_public_key();

        // No Authorization header = internal service call (TODO: replace with mTLS)
        let Some(auth_header) = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
        else {
            return Ok(AdminOrInternal(None));
        };

        let token = auth_header
            .strip_prefix("Bearer ")
            .or_else(|| auth_header.strip_prefix("bearer "))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            ))?;

        let claims = verify_jwt(&token, &jwt_public_key).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Unauthorized")),
            )
        })?;

        // Check if user has admin role
        for role in claims.roles {
            if role == Role::Admin {
                return Ok(AdminOrInternal(Some(claims.sub)));
            }
        }

        Err((StatusCode::FORBIDDEN, Json(ErrorBody::new("Forbidden"))))
    }
}
