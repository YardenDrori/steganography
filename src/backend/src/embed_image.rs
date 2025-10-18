use image::{self, RgbaImage};
use std::fs::{self, File};
use std::io::Read;

// const CARRIER: &str = "../../examples/images/output.png";
// // const CARRIER: &str = "../../examples/images/solid_white.png";
const CARRIER: &str = "../../examples/images/png_image.png";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";
const STEGO_FILE: &str = "../../output/stego_files/stego_file.png";

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read carrier file
    let frame = image::open(CARRIER)?.into_rgba8();
    let dimensions = frame.dimensions();
    println!("{:?}", dimensions);

    let chunk_size = dimensions.0 * dimensions.1 * 4;
    //check if carrier has capacity for payload
    if fs::metadata(PAYLOAD)?.len() > chunk_size as u64 {
        return Err("Payload is too large to fit in the carrier file".into());
    }
    //buffer is same size as image to make one bugger fit one image exactly
    let mut buffer = vec![0; chunk_size as usize];

    //read frame into memory
    let mut pixels = frame.into_raw();
    println!("{:?}\n", &pixels[0..8]);
    let mut payload = File::open(PAYLOAD)?;
    let bytes_read = payload.read(&mut buffer)?;

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

    output_image.save(STEGO_FILE)?;

    Ok(())
}
