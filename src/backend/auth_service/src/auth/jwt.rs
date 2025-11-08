use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // subject (user_id)
    pub exp: i64, // expiration timestamp in unix convention
    pub iat: i64, // issued at timestamp in unix convention
}

// Low-level JWT encoding utility
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

// Low-level JWT decoding utility
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let token_data = decode::<Claims>(
        token,
        &decoding_key,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}
