mod embed_image;
mod embed_video;

// const CARRIER: &str = "../../examples/images/output.png";
// // const CARRIER: &str = "../../examples/images/solid_white.png";
const CARRIER: &str = "../../examples/images/png_image.png";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // embed_image::embed_image(CARRIER, PAYLOAD)?;
    Ok(())
}
