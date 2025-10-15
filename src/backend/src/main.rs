use image::{self};
use std::fs::{self};

const IMAGE_FILE: &str = "../../examples/images/png_image.png";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read carrier file
    let frame = image::open(IMAGE_FILE)?.into_rgba8();
    let dimensions = frame.dimensions();
    println!("{:?}", dimensions);

    //check if carrier has capacity for payload
    if fs::metadata(PAYLOAD)?.len() > dimensions.0 as u64 * dimensions.1 as u64 * 4 {
        return Err("Payload is too large to fit in the carrier file".into());
    }
    //read payload
    let payload = fs::read(PAYLOAD);

    let pixels = frame.as_raw();
    println!("{:?}", &pixels[0..3]);

    Ok(())
}
