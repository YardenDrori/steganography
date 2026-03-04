use serde::{Deserialize, Serialize};
pub use shared_global::dtos::UserResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YUVChannels {
    pub y: bool,
    pub cb: bool,
    pub cr: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RGBChannels {
    pub r: bool,
    pub g: bool,
    pub b: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channels {
    pub yuv: Option<YUVChannels>,
    pub rgb: Option<RGBChannels>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedConfigs {
    pub channels_to_embed: Channels,
    pub coefficients_to_embed: [bool; 16],
    pub delta: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedFileRequest {
    pub payload_id: i64,
    pub carrier_id: i64,
    pub configs: EmbedConfigs,
}
