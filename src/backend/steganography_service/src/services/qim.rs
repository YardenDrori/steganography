#[rustfmt::skip]
pub const ZIGZAG: [usize ; 16] = [
0, 1, 4, 8, 5, 2, 3, 6, 9, 12, 13, 10, 7, 11, 14, 15,
];

// bit=true  → nearest multiple of delta
// bit=false → nearest odd multiple of delta/2
pub fn qim_embed(mut block: [f64; 16], bit: bool, delta: u8, zigzag_index: usize) -> [f64; 16] {
    let delta_float = delta as f64;
    let flat_index = ZIGZAG[zigzag_index];
    let coeff = block[flat_index];

    block[flat_index] = if bit {
        (coeff / delta_float).round() * delta_float
    } else {
        ((coeff / delta_float - 0.5).round() + 0.5) * delta_float
    };

    block
}

// Reads the embedded bit from a DCT coefficient.
pub fn qim_extract(block: [f64; 16], delta: u8, zigzag_index: usize) -> bool {
    let delta_float = delta as f64;
    let flat_index = ZIGZAG[zigzag_index];
    let coeff = block[flat_index];

    let dist_true = (coeff - (coeff / delta_float).round() * delta_float).abs();
    let dist_false = (coeff - ((coeff / delta_float - 0.5).round() + 0.5) * delta_float).abs();

    dist_true < dist_false
}
