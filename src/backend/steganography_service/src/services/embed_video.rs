use std::path::PathBuf;

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    Ok(())
}
