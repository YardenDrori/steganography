use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, Validation, decode, encode};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};

const ACCESS_TOKEN_DURATION: usize = 5 * 60; //5 mins
const REFRESH_TOKEN_DURATION: usize = 30 * 24 * 60 * 60; //14 days

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,   //subject (user_id)
    pub exp: usize, //expiration timestamp in unix convention
    pub iat: usize, //implemented at timestamp in unix convention
}

//JWT methods
pub fn create_jwt(user_id: i64, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(&secret.as_bytes());
    let now = Utc::now();
    let iat = now.timestamp() as usize; //convert to unix timestamp
    let exp = iat + ACCESS_TOKEN_DURATION;

    let claims: Claims = Claims {
        sub: user_id,
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

//REFRESH_TOKEN methods
const ALPHANUMERIC_BINARY_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const TOKEN_LEN: usize = 64;
pub async fn create_refresh_token() -> String {
    let mut rand = rand::rng();

    let token: String;
    token.reserve_exact(TOKEN_LEN);
    for i in [0..TOKEN_LEN] {
        let char_index = r
        token.push_str();
    }

    todo!()
}
