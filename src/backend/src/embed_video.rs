use ffmpeg_next::{self as ffmpeg, codec::video};

fn embed_video(
    carrier_location: &str,
    payload_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    //input context - handle for the video
    let mut ictx = ffmpeg::format::input(carrier_location)?;
    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

    let video_stream_index = input_stream.index();
    //metadata for video stream + decoder configs
    let decoder_context =
        ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;

    //video decoder
    let mut decoder = decoder_context.decoder().video()?;

    for (stream, packet) in ictx.packets() {
        if (stream.index() == video_stream_index) {
            //sends packet to decoder one packet might not be 1 frame so it doesnt return an iput
            //as soon as it is called
            decoder.send_packet(&packet)?;

            let mut frame = ffmpeg::util::frame::video::Video::empty();
            //keep annoying the decoder untill it gets off its fat ass and gives a frame
            while decoder.receive_frame(&mut frame).is_ok() {}
        }
    }
}
