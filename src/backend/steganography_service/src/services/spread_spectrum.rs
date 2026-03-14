use crate::errors::steg_service_error::StegServiceError;
use crate::services::vector;

pub fn spread_spectrum_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;

    let bit_to_embed_mult: f64 = if bit_to_embed { 1f64 } else { -1f64 };

    for i in 0..coeff_count {
        let curr_coeff = get_coeff(i)?;
        let modified_coeff = curr_coeff
            + (unit_vector
                .get(i)
                .ok_or(StegServiceError::CollectionCallWithInvalidKey)?
                * bit_to_embed_mult
                * delta as f64);
        set_coeff(i, modified_coeff)?;
    }

    Ok(())
}

pub fn spread_spectrum_extract(
    _get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    _coeff_count: usize,
    _seed: String,
    _delta: u8,
) -> Result<bool, StegServiceError> {
    todo!()
}
