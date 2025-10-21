mod embed_image;
mod embed_video;
mod extract_image;
mod extract_video;

const CARRIER: &str = "../../examples/videos_lossless/chicken_jockey.mkv";
const PAYLOAD: &str = "../../examples/hideable_files/bee_movie_script.txt";
const STEG_FILE: &str = "../../output/steg_files/output.mkv";
const EXTRACTED_PAYLOAD: &str = "../../output/payloads/output";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    embed_video::embed_video(CARRIER, PAYLOAD, STEG_FILE)?;
    extract_video::extract_video(STEG_FILE, EXTRACTED_PAYLOAD)?;
    Ok(())
}
