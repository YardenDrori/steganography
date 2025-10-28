use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

const ACCESS_TOKEN_DURATION: usize = 15 * 60; //15 mins

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, //subject (user_id)
    pub exp: usize,  //expiration timestamp in unix convention
    pub iat: usize,  //implemented at timestamp in unix convention
}

pub fn create_jwt(user_id: i64, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(&secret.as_bytes());

    let now = Utc::now();
    let iat = now.timestamp() as usize; //convert to unix timestamp
    let exp = iat + ACCESS_TOKEN_DURATION;

    let claims: Claims = Claims {
        sub: user_id.to_string(),
        exp: exp,
        iat,
    };

    let signature = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &encoding_key,
    )?;

    Ok(signature)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let claims = decode::<Claims>(
        token,
        &decoding_key,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;

    Ok(claims.claims)
}
