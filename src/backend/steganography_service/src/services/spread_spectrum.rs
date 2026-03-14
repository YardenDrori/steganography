use crate::errors::steg_service_error::StegServiceError;
use crate::services::vector::{self, calculate_dot_product};

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
        let curr_unit_vec_mult = unit_vector
            .get(i)
            .ok_or(StegServiceError::CollectionCallWithInvalidKey)?;
        let embedded_coeff = curr_coeff + (bit_to_embed_mult * delta as f64) * curr_unit_vec_mult;

        set_coeff(i, embedded_coeff)?;
    }

    Ok(())
}

//ISS (not international space station😭)
pub fn improved_spread_spectrum_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;

    let bit_to_embed_mult: f64 = if bit_to_embed { 1f64 } else { -1f64 };
    let dot_product = calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;

    for i in 0..coeff_count {
        let curr_coeff = get_coeff(i)?;
        let curr_unit_vec_mult = unit_vector
            .get(i)
            .ok_or(StegServiceError::CollectionCallWithInvalidKey)?;
        let embedded_coeff =
            curr_coeff + ((bit_to_embed_mult * delta as f64) - dot_product) * curr_unit_vec_mult;

        set_coeff(i, embedded_coeff)?;
    }

    Ok(())
}

pub fn spread_spectrum_extract(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    coeff_count: usize,
    seed: String,
) -> Result<bool, StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;

    let dot_product = calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;

    //NOTE: it is tehcnically possible tho astronimically impossible that the value is exactly 0 if
    //thats the case were fucked for that bit lol we literally have no way to figure what it was
    //and we might as well just guess which is exactly what we do here we have ECC for a reason!
    Ok(dot_product > 0f64)
}
