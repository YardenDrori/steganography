mod embed_image;
mod embed_video;
mod extract_image;

const CARRIER: &str = "../../examples/videos_lossless/chicken_jockey.mkv";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", embed_video::embed_video(CARRIER, PAYLOAD)?);
    Ok(())
}
