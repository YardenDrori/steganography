use std::any::Any;

use crate::{
    errors::steg_service_error::StegServiceError,
    services::{
        qim::{qim_embed, qim_extract_bit},
        spread_spectrum::{
            improved_spread_spectrum_embed, spread_spectrum_embed, spread_spectrum_extract,
        },
        stdm::{stdm_embed, stdm_extract},
    },
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
    pub seed: Option<String>,
    pub method: EmbedMethods,
}

impl EmbedConfigs {
    pub fn validate_configs(&self) -> Result<(), StegServiceError> {
        if self.coefficients_per_bit < 1 {
            return Err(StegServiceError::InvalidPayload);
        }

        let mut found_coeff = false;
        for i in 0..16 {
            if self.coefficients_to_embed[i] == true {
                found_coeff = true;
                break;
            }
        }
        if !found_coeff {
            return Err(StegServiceError::InvalidPayload);
        }

        if matches!(self.method, EmbedMethods::QIM) && self.coefficients_per_bit != 1 {
            return Err(StegServiceError::InvalidPayload);
        }
        Ok(())
    }
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
        seed: Option<String>,
        bit: bool,
        delta: u8,
    ) -> Result<(), StegServiceError> {
        match self {
            EmbedMethods::QIM => {
                qim_embed(get_coeff, set_coeff, bit, delta)?;
                Ok(())
            }
            EmbedMethods::STDM => {
                stdm_embed(
                    get_coeff,
                    set_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    bit,
                    delta,
                )?;
                Ok(())
            }
            EmbedMethods::SS => {
                spread_spectrum_embed(
                    get_coeff,
                    set_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    bit,
                    delta,
                )?;
                Ok(())
            }
            EmbedMethods::ISS => {
                improved_spread_spectrum_embed(
                    get_coeff,
                    set_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    bit,
                    delta,
                )?;
                Ok(())
            }
        }
    }

    pub fn extract(
        self,
        get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
        coeff_count: usize,
        seed: Option<String>,
        delta: u8,
    ) -> Result<bool, StegServiceError> {
        match self {
            Self::QIM => {
                let coeff = get_coeff(0)?;
                let found_bit = qim_extract_bit(coeff, delta);
                return Ok(found_bit);
            }
            Self::STDM => {
                let found_bit = stdm_extract(
                    get_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    delta,
                )?;
                return Ok(found_bit);
            }
            //since the only difference between SS and ISS is the embed step both use the same
            //extractor for decoding
            Self::SS => {
                let found_bit = spread_spectrum_extract(
                    get_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                )?;
                return Ok(found_bit);
            }
            Self::ISS => {
                let found_bit = spread_spectrum_extract(
                    get_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                )?;
                return Ok(found_bit);
            }
        };
    }
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
