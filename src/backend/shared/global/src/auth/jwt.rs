use crate::auth::roles::Roles;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

pub trait HasJwtPublicKey {
    fn jwt_public_key(&self) -> String;
}

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // subject (user_id)
    pub exp: i64, // expiration timestamp in unix convention
    pub iat: i64, // issued at timestamp in unix convention
    pub roles: Roles,
}

pub fn verify_jwt(token: &str, public_key_pem: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
