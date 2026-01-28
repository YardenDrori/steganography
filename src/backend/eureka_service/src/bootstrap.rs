use rsa::pkcs8::EncodePrivateKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::pkcs8::LineEnding;
use rsa::RsaPrivateKey;
use sqlx::{Pool, Postgres};

pub struct SharedConfig {
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub internal_api_key: String,
}

pub async fn bootstrap_config(
    pool: &Pool<Postgres>,
) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let existing = sqlx::query_as!(
        SharedConfig,
        "SELECT jwt_private_key, jwt_public_key, internal_api_key FROM shared_config WHERE id = 1"
    )
    .fetch_optional(pool)
    .await?;

    if let Some(config) = existing {
        tracing::info!("Loaded existing shared config from database");
        return Ok(config);
    }

    tracing::info!("First boot detected - generating RSA keypair and internal API key");

    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = rsa::RsaPublicKey::from(&private_key);

    let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF)?;
    let public_key_pem = public_key.to_public_key_pem(LineEnding::LF)?;

    let api_key_bytes: [u8; 32] = rand::Rng::gen(&mut rng);
    let internal_api_key = hex::encode(api_key_bytes);

    sqlx::query!(
        "INSERT INTO shared_config (jwt_private_key, jwt_public_key, internal_api_key) VALUES ($1, $2, $3)",
        private_key_pem.as_str(),
        &public_key_pem,
        &internal_api_key,
    )
    .execute(pool)
    .await?;

    tracing::info!("Generated and stored new shared config");

    Ok(SharedConfig {
        jwt_private_key: private_key_pem.to_string(),
        jwt_public_key: public_key_pem,
        internal_api_key,
    })
}
