use crate::app_state::AppState;
use axum::extract::State;
use chrono::Utc;
use jsonwebtoken::{decode, encode, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ATTEMPTS: u8 = 3;
