use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use super::roles::Roles;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,      // subject (user_id)
    pub exp: i64,      // expiration timestamp in unix convention
    pub iat: i64,      // issued at timestamp in unix convention
    pub roles: Roles,  // user roles for authorization
}

/// Low-level JWT decoding utility
/// Used by gateway and all services to verify JWT signatures
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let token_data = decode::<Claims>(
        token,
        &decoding_key,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}
