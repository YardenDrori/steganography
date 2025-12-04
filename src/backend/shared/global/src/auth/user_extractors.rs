use crate::auth::roles::Role;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};

use crate::{
    auth::jwt::{verify_jwt, HasJwtPublicKey},
    errors::ErrorBody,
};

/// Extractor for authenticated user from API Gateway headers
/// Gateway adds X-User-Id header after verifying JWT
pub struct AuthenticatedUser(pub i64);
pub struct RequireAdmin(pub i64);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + HasJwtPublicKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_public_key = state.jwt_public_key();
        let headers = parts
            .headers
            .get("authorization")
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("No authorization header found")),
            ))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Invalid Authorization header format")),
                )
            })?;

        let token = headers
            .strip_prefix("Bearer ")
            .or_else(|| headers.strip_prefix("bearer "))
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Intenral server error")),
            ))?;

        let claims = verify_jwt(&token, &jwt_public_key).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Invalid or expired token")),
            )
        })?;
        let user = AuthenticatedUser { 0: claims.sub };
        Ok(user)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync + HasJwtPublicKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_public_key = state.jwt_public_key();
        let headers = parts
            .headers
            .get("authorization")
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("No authorization header found")),
            ))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody::new("Invalid Authorization header format")),
                )
            })?;

        let token = headers
            .strip_prefix("Bearer ")
            .or_else(|| headers.strip_prefix("bearer "))
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody::new("Intenral server error")),
            ))?;

        let claims = verify_jwt(&token, &jwt_public_key).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Invalid or expired token")),
            )
        })?;

        for role in claims.roles {
            if role == Role::Admin {
                return Ok(RequireAdmin { 0: claims.sub });
            }
        }

        return Err((StatusCode::FORBIDDEN, Json(ErrorBody::new("Forbidden"))));
    }
}
