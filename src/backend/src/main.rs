mod embed_image;
mod embed_video;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    embed_video::embed_video();
    Ok(())
}
