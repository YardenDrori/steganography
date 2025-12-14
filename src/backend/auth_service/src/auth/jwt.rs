use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use shared_global::auth::{jwt::Claims, roles::Roles};

pub fn encode_jwt(
    user_id: i64,
    issued_at: i64,
    expires_at: i64,
    roles: Roles,
    private_key_pem: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;

    let claims = Claims {
        sub: user_id,
        exp: expires_at,
        iat: issued_at,
        roles: roles,
    };

    encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
}
