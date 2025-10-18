use image::{self, ImageBuffer, Rgba, RgbaImage};
use std::fs::{self, File};
use std::io::Read;

pub fn embed_image(
    carrier: &mut [u8],
    payload: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> Result<usize, Box<dyn std::error::Error>> {
    //read carrier file
    // let frame = image::open(carrier)?.into_rgba8();
    // let dimensions = frame.dimensions();
    // println!("{:?}", dimensions);
    //
    // //we redyce by 32 as we mark the first 32 bits (1 byte) for the size of the payload to know how much of
    // //the file we need to read incase the payload is smaller then the carrier
    // let chunk_size = dimensions.0 * dimensions.1 * 4 - 32;
    // //check if carrier has capacity for payload
    // if fs::metadata(payload)?.len() > chunk_size as u64 {
    //     return Err("Payload is too large to fit in the carrier file".into());
    // }
    // //buffer is same size as image to make one bugger fit one image exactly
    // let mut buffer = vec![0; chunk_size as usize];
    //
    // //read frame into memory
    // let mut pixels = frame.into_raw();
    // println!("{:?}\n", &pixels[0..8]);
    // let mut payload = File::open(payload)?;
    // let bytes_read = payload.read(&mut buffer)?;

    //insert patload byte size
    let payload_len = payload.len(); //in bytes
    for i in 0..32 {
        carrier[i] = carrier[i] & 0xFE; //set LSB to 0
    }

    //embed payload
    let chunk_data = &mut buffer[0..bytes_read];
    let mut chunk_byte = 0;
    for pixel in pixels.iter_mut() {
        if chunk_byte >= bytes_read {
            break;
        }
        for _i in 0..8 {
            *pixel = (*pixel & 0xFE) | (chunk_data[chunk_byte] & 1);
            chunk_data[chunk_byte] = chunk_data[chunk_byte] >> 1;
        }
        chunk_byte += 1;
    }
    let output_image = RgbaImage::from_vec(dimensions.0, dimensions.1, pixels)
        .ok_or("Failed to create image from raw pixels")?;

    //debug line to save frame
    output_image.save("../../output/stego_files/stego_file.png")?;

    Ok()
}
