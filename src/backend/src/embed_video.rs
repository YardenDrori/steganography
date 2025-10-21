use crate::embed_image;
use std::fs::File;
use std::io::Read;

use ffmpeg_next::{self as ffmpeg, codec::video, format};

fn embed_video(
    carrier_location: &str,
    payload_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    ffmpeg::init()?;
    //handle for payload
    let payload_handle = File::open(payload_location);
    let payload_buffer: Vec<u8> = vec![0];

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

    //get the frame data from individual packets
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            //sends packet to decoder one packet might not be 1 frame so it doesnt return an iput
            //as soon as it is called
            decoder.send_packet(&packet)?;

            let mut frame = ffmpeg::util::frame::video::Video::empty();
            //keep nagging the decoder untill it gets off its fat ass and gives a frame
            while decoder.receive_frame(&mut frame).is_ok() {
                //some formats work differently rgb 8 has one array of 8 bits in each slot while
                //yuv420 has 3 seperate arrays with 4 times as many 'y'
                let channel_count = frame.planes();
                let payload_offset: usize = 0;
                for channel_index in 0..channel_count {
                    let channel_data = frame.data_mut(channel_index);
                }
            }
        }
    }
    Ok("rock and stone".to_string())
}
