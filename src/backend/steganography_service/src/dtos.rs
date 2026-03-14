use crate::{
    errors::steg_service_error::StegServiceError, services::spread_spectrum::spread_spectrum_embed,
};
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
    pub coefficients_per_bit: usize,
    pub delta: u8,
    pub seed: String,
    pub method: EmbedMethods,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmbedMethods {
    QIM,
    STDM,
    SS,
    ISS,
}
//generic embed method yipee!
impl EmbedMethods {
    pub fn embed(
        &self,
        get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
        set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
        coeff_count: usize,
        seed: String,
        bit: bool,
        delta: u8,
    ) -> Result<(), StegServiceError> {
        match self {
            EmbedMethods::QIM => {
                qim::embed(todo!());
                Ok(())
            }
            EmbedMethods::STDM => {
                stdm::embed(todo!());
                Ok(())
            }
            EmbedMethods::SS => {
                spread_spectrum_embed(get_coeff, set_coeff, coeff_count, seed, bit, delta)?;
                Ok(())
            }
            EmbedMethods::ISS => {
                iss::embed(todo!());
                Ok(())
            }
        }
    }
    // pub fn extract()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedFileRequest {
    pub payload_id: i64,
    pub carrier_id: i64,
    pub configs: EmbedConfigs,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractFileRequest {
    pub steg_object_id: i64,
    pub configs: EmbedConfigs,
}
