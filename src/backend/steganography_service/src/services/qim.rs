use crate::errors::steg_service_error::StegServiceError;

#[rustfmt::skip]
pub const ZIGZAG: [(usize, usize); 16] = [
    (0, 0), (0, 1), (1, 0), (2, 0),
    (1, 1), (0, 2), (0, 3), (1, 2),
    (2, 1), (3, 0), (3, 1), (2, 2),
    (1, 3), (2, 3), (3, 2), (3, 3),
];

// bit=true  → nearest multiple of delta
// bit=false → nearest odd multiple of delta/2
pub fn qim_embed(
    mut block: [[f64; 4]; 4],
    bit: bool,
    delta: u8,
    zigzag_index: usize,
) -> Result<[[f64; 4]; 4], StegServiceError> {
    let delta_float = delta as f64;
    let (x, y) = ZIGZAG[zigzag_index];
    let coeff = block[x][y];

    block[x][y] = if bit {
        (coeff / delta_float).round() * delta_float
    } else {
        ((coeff / delta_float - 0.5).round() + 0.5) * delta_float
    };

    Ok(block)
}

// Reads the embedded bit from a DCT coefficient.
pub fn qim_extract(block: [[f64; 4]; 4], delta: u8, zigzag_index: usize) -> bool {
    let delta_float = delta as f64;
    let (x, y) = ZIGZAG[zigzag_index];
    let coeff = block[x][y];

    let dist_true = (coeff - (coeff / delta_float).round() * delta_float).abs();
    let dist_false = (coeff - ((coeff / delta_float - 0.5).round() + 0.5) * delta_float).abs();

    dist_true < dist_false
}
