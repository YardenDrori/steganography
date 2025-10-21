pub fn embed_image(
    carrier: &mut [u8],
    payload: &[u8],
    is_10_bit_format: bool, //not implemented as of yet
) -> Result<usize, Box<dyn std::error::Error>> {
    //insert patload byte size in first 32 pixels
    let payload_len = payload.len() as u32; //in bytes
    for pixel in 0..32 {
        let bit = (payload_len >> (31 - pixel)) & 1;
        carrier[pixel] = (carrier[pixel] & 0xFE) | bit as u8;
    }

    //embed data in image
    let mut pixel_index = 32;
    let mut bytes_embedded = 0;
    for byte in payload {
        for i in (0..8).rev() {
            carrier[pixel_index] = (carrier[pixel_index] & 0xFE) | ((byte >> i) & 1);
            pixel_index += 1;
        }
        bytes_embedded += 1;
    }

    Ok(bytes_embedded as usize)
}
