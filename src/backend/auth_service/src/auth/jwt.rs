use jsonwebtoken::{encode, EncodingKey, Header};
use shared_global::auth::jwt::Claims;

/// Low-level JWT encoding utility
/// Only used by auth_service to create tokens
pub fn encode_jwt(
    user_id: i64,
    issued_at: i64,
    expires_at: i64,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    let claims = Claims {
        sub: user_id,
        exp: expires_at,
        iat: issued_at,
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &encoding_key,
    )
}
