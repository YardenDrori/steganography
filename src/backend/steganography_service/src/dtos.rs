use serde::{Deserialize, Serialize};
pub use shared_global::dtos::UserResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channels {
    pub y: bool,
    pub cb: bool,
    pub cr: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedConfigs {
    pub channels_to_embed: Channels,
    pub bits_per_block: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedFileRequest {
    pub payload_id: i64,
    pub carrier_id: i64,
    pub configs: EmbedConfigs,
}
