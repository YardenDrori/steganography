use axum::extract::State;
use chrono::Utc;
use jsonwebtoken::{decode, encode, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app_state::AppState;

const ACCESS_TOKEN_DURATION: i64 = 10 * 60; //10 mins
const REFRESH_TOKEN_DURATION: i64 = 14 * 24 * 60 * 60; //14 days
const REFRESH_TOKEN_LEN: u8 = 64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, //subject (user_id)
    pub exp: i64, //expiration timestamp in unix convention
    pub iat: i64, //implemented at timestamp in unix convention
}

//JWT methods
pub fn create_jwt(user_id: i64, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(&secret.as_bytes());
    let now = Utc::now();
    let iat = now.timestamp(); //convert to unix timestamp
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
pub async fn create_refresh_token(
    State(app_state): State<AppState>,
    user_id: i64,
) -> Result<String, sqlx::Error> {
    todo!()
}

// let pool = &app_state.pool;
// let mut rand = rand::rng();
//
// //if SOMEHOW the random key generated was already in use 3 times in a
// //row we return the sqlx error
// for attempt in 0..ATTEMPTS {
//     //generate random key
//     let token: String = (0..TOKEN_LEN)
//         .map(|_| {
//             let index = rand.random_range(0..ALPHANUMERIC_BINARY_CHARS.len());
//             ALPHANUMERIC_BINARY_CHARS[index] as char
//         })
//         .collect();
//     let token_hash = format!("{:?}", Sha256::digest(&token));
//
//     let time_delta = chrono::TimeDelta::seconds(REFRESH_TOKEN_DURATION);
//     let expiration_time = Utc::now().checked_add_signed(time_delta);
//
//     let result = sqlx::query!(
//         r#"
//         INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
//         VALUES ($1, $2, $3) RETURNING id
//         "#,
//         user_id,
//         token_hash,
//         expiration_time
//     )
//     .fetch_one(pool)
//     .await?;
// }
//
// unreachable!("Should have returned or panicked in the loop")
