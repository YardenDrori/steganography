use image::{self, DynamicImage, GenericImage, ImageBuffer, RgbaImage};
use std::fs::{self, File};
use std::io::{BufReader, Read};

const CARRIER: &str = "../../examples/images/png_image.png";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read carrier file
    let frame = image::open(CARRIER)?.into_rgba8();
    let dimensions = frame.dimensions();
    println!("{:?}", dimensions);

    //make empty image with same resolution idk if needed yet prob better to edit existing one
    let mut new_image = RgbaImage::new(dimensions.0, dimensions.1);

    let chunk_size = dimensions.0 * dimensions.1 * 4;
    //check if carrier has capacity for payload
    if fs::metadata(PAYLOAD)?.len() > chunk_size as u64 {
        return Err("Payload is too large to fit in the carrier file".into());
    }
    //buffer is same size as image to make one bugger fit one image exactly
    let mut buffer = vec![0; chunk_size as usize];

    //read frame into memory
    let pixels = frame.as_raw();
    println!("{:?}", &pixels[0..3]); //read payload as chunks 
    let mut payload = File::open(PAYLOAD)?;
    loop {
        let bytes_read = payload.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let chunk_data = &buffer[0..bytes_read];
    }

    Ok(())
}
