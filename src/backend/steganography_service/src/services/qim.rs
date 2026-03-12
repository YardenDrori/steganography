// bit=true  → nearest multiple of delta
// bit=false → nearest odd multiple of delta/2
pub fn qim_embed(coeff: f64, bit: bool, delta: u8) -> f64 {
    let delta_float = delta as f64;

    if bit {
        return (coeff / delta_float).round() * delta_float;
    } else {
        return ((coeff / delta_float - 0.5).round() + 0.5) * delta_float;
    }
}

// Reads the embedded bit from a DCT coefficient.
pub fn qim_extract(coeff: f64, delta: u8) -> bool {
    let delta_float = delta as f64;

    let dist_true = (coeff - (coeff / delta_float).round() * delta_float).abs();
    let dist_false = (coeff - ((coeff / delta_float - 0.5).round() + 0.5) * delta_float).abs();

    dist_true < dist_false
}
