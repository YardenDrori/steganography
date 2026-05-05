use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use aes_gcm::aead::rand_core::RngCore;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use crate::errors::steg_service_error::StegServiceError;

const PBKDF2_ITERATIONS: u32 = 200_000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

pub fn aes_encrypt(password: &str, plaintext: &[u8]) -> Result<Vec<u8>, StegServiceError> {
    if password.is_empty() {
        return Ok(plaintext.to_vec());
    }

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| StegServiceError::FileError)?;

    let mut out = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn aes_decrypt(password: &str, blob: &[u8]) -> Result<Vec<u8>, StegServiceError> {
    if password.is_empty() {
        return Ok(blob.to_vec());
    }

    // minimum: 16 salt + 12 nonce + 16 GCM tag
    if blob.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err(StegServiceError::DecryptionError);
    }

    let salt = &blob[..SALT_LEN];
    let nonce_bytes = &blob[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext_with_tag = &blob[SALT_LEN + NONCE_LEN..];

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext_with_tag)
        .map_err(|_| StegServiceError::DecryptionError)
}
