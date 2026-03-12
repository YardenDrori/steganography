use crate::errors::steg_service_error::StegServiceError;
use crate::services::qim::qim_extract;
use crate::services::{qim, vector};

pub fn stdm_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;

    let original_dot_product =
        vector::calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;

    let embedded_dot_product = qim::qim_embed(original_dot_product, bit_to_embed, delta);

    vector::do_back_projection_on_coeffs(
        get_coeff,
        set_coeff,
        coeff_count,
        &unit_vector,
        original_dot_product,
        embedded_dot_product,
    )?;

    Ok(())
}

pub fn stdm_extract(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    coeff_count: usize,
    seed: String,
    delta: u8,
) -> Result<bool, StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;

    let dot_product = vector::calculate_dot_product(get_coeff, coeff_count, &unit_vector)?;

    let bit = qim_extract(dot_product, delta);
    Ok(bit)
}
