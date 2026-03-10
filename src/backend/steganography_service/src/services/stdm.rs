use crate::errors::steg_service_error::StegServiceError;
use crate::services::{qim, vector};

pub fn stdm_embed(
    coeffs: &mut [f64],
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeffs.len())?;

    let original_dot_product = vector::calculate_dot_product(coeffs, &unit_vector)?;

    let embedded_dot_product = qim::qim_embed(original_dot_product, bit_to_embed, delta);

    vector::do_back_projection_on_coeffs(
        coeffs,
        &unit_vector,
        original_dot_product,
        embedded_dot_product,
    )?;

    Ok(())
}
