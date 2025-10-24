pub fn extract_from_image(
    steg_file_bin: &mut [u8],
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    //find the length of the payload in the file
    let mut payload_byte_count: usize = 0;
    for i in 0..32 {
        payload_byte_count = payload_byte_count << 1;
        payload_byte_count = (steg_file_bin[i] as usize & 1) | payload_byte_count;
    }

    if payload_byte_count == 0 {
        return Ok(None);
    }

    let mut payload_bin: Vec<u8> = vec![0; payload_byte_count];

    //extract the payload
    let mut pixel_index: usize = 32;
    for i in 0..payload_byte_count {
        for _bit in 0..8 {
            payload_bin[i] = payload_bin[i] << 1;
            payload_bin[i] = (steg_file_bin[pixel_index] & 1) | payload_bin[i];
            pixel_index += 1;
        }
    }

    Ok(Some(payload_bin))
}
