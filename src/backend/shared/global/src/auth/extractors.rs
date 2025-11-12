use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use sqlx::PgPool;

use crate::db::pool_provider::HasPgPool;

/// Extractor for authenticated user from API Gateway headers
/// Gateway adds X-User-Id header after verifying JWT
pub struct AuthenticatedUser(pub i64);
pub struct RequireAdmin(pub i64);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = parts
            .headers
            .get("X-User-Id")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let user = AuthenticatedUser {
            0: headers
                .parse::<i64>()
                .map_err(|_| StatusCode::BAD_REQUEST)?,
        };
        Ok(user)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync + HasPgPool,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let headers = parts
            .headers
            .get("X-User-Id")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let user = RequireAdmin {
            0: headers
                .parse::<i64>()
                .map_err(|_| StatusCode::BAD_REQUEST)?,
        };
        Ok(user)
    }
}
