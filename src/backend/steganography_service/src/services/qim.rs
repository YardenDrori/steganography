use crate::errors::steg_service_error::StegServiceError;

pub fn qim_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let curr_coeff = get_coeff(0)?;
    let embedded_coeff = qim_embed_bit(curr_coeff, bit_to_embed, delta);
    set_coeff(0, embedded_coeff)?;
    Ok(())
}

// bit=true  → nearest multiple of delta
// bit=false → nearest odd multiple of delta/2
pub fn qim_embed_bit(coeff: f64, bit: bool, delta: u8) -> f64 {
    let delta_float = delta as f64;

    if bit {
        return (coeff / delta_float).round() * delta_float;
    } else {
        return ((coeff / delta_float - 0.5).round() + 0.5) * delta_float;
    }
}

// Reads the embedded bit from a DCT coefficient.
pub fn qim_extract_bit(coeff: f64, delta: u8) -> bool {
    let delta_float = delta as f64;

    let dist_true = (coeff - (coeff / delta_float).round() * delta_float).abs();
    let dist_false = (coeff - ((coeff / delta_float - 0.5).round() + 0.5) * delta_float).abs();

    dist_true < dist_false
}
