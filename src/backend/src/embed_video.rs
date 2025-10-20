use ffmpeg_next as ffmpeg;

fn embed_video(
    carrier_location: &str,
    payload_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    //input context - handle for the video
    let mut ictx = ffmpeg::format::input(carrier_location)?;
    ictx.streams().best(Type::Video);
}
