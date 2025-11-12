use jsonwebtoken::{encode, EncodingKey, Header};
use shared_global::auth::{jwt::Claims, roles::Roles};

/// Low-level JWT encoding utility
/// Only used by auth_service to create tokens
pub fn encode_jwt(
    user_id: i64,
    issued_at: i64,
    expires_at: i64,
    roles: Roles,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    let claims = Claims {
        sub: user_id,
        exp: expires_at,
        iat: issued_at,
        roles: roles,
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &encoding_key,
    )
}
