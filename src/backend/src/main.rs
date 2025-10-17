use image::{self, ImageBuffer};
use std::fs::{self, File};
use std::io::{BufReader, Read};

const CARRIER: &str = "../../examples/images/png_image.png";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read carrier file
    let frame = image::open(CARRIER)?.into_rgba8();
    let dimensions = frame.dimensions();
    println!("{:?}", dimensions);
    let mut new_image = ImageBuffer::new(dimensions.0, dimensions.1);

    //check if carrier has capacity for payload
    if fs::metadata(PAYLOAD)?.len() > dimensions.0 as u64 * dimensions.1 as u64 * 4 {
        return Err("Payload is too large to fit in the carrier file".into());
    }
    let chunk_size = dimensions.0 * dimensions.1 * 4;
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
