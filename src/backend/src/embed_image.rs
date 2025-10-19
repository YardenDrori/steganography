pub fn embed_image(
    carrier: &mut [u8],
    payload: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> Result<usize, Box<dyn std::error::Error>> {
    //insert patload byte size in first 32 pixels
    let payload_len = payload.len() as u32; //in bytes
    for pixel in 0..32 {
        let bit = (payload_len >> (31 - pixel)) & 1;
        carrier[pixel] = (carrier[pixel] & 0xFE) | bit as u8;
    }

    // Calculate total available bits for payload (excluding the first 32 pixels used for length)
    let total_pixels = (width * height * channels as u32) as usize;
    let available_bits = total_pixels - 32; // First 32 pixels are reserved for length
    let available_bytes = available_bits / 8;
    if available_bytes < payload.len() {
        return Err(format!(
            "Payload is too large ({} bytes) for carrier's capacity ({} bytes).",
            available_bytes,
            payload.len()
        )
        .into());
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

    // let output_image = RgbaImage::from_vec(dimensions.0, dimensions.1, pixels)
    // .ok_or("Failed to create image from raw pixels")?;

    //debug line to save frame
    // output_image.save("../../output/stego_files/stego_file.png")?;

    Ok(bytes_embedded as usize)
}
