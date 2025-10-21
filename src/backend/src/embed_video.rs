use crate::embed_image;
use std::fs::File;
use std::io::Read;

use ffmpeg_next::{self as ffmpeg};

pub fn embed_video(
    carrier_location: &str,
    payload_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    let mut payload_file = File::open(payload_location)?;
    let mut payload_bytes_embedded = 0;

    // === DECODING ===
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

    // === ENCODING ===

    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::FFV1);
    //output context
    let mut octx = ffmpeg::format::output("output.mkv")?;

    let mut output_stream = octx.add_stream(codec)?;

    let mut encoder_context =
        ffmpeg::codec::context::Context::new_with_codec(codec.ok_or(ffmpeg::Error::InvalidData)?)
            .encoder()
            .video()?;

    //not sure we need it in examples they had this line twice so for now ill coppy test it later
    output_stream.set_parameters(&encoder_context);
    //coppy the properties of the decoder
    encoder_context.set_height(decoder.height());
    encoder_context.set_width(decoder.width());
    encoder_context.set_aspect_ratio(decoder.aspect_ratio());
    encoder_context.set_format(decoder.format());
    encoder_context.set_frame_rate(decoder.frame_rate());
    encoder_context.set_time_base(input_stream.time_base());
    let mut encoder = encoder_context.open_as(codec)?;
    output_stream.set_parameters(&encoder);
    octx.write_header()?;

    let mut payload_exhausted = false;

    //get the frame data from individual packets
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            //sends packet to decoder one packet might not be 1 frame so it doesnt return an iput
            //as soon as it is called
            decoder.send_packet(&packet)?;

            let mut frame = ffmpeg::util::frame::video::Video::empty();
            //keep nagging the decoder untill it gets off its fat ass and gives a frame
            while decoder.receive_frame(&mut frame).is_ok() {
                process_frame(
                    &mut frame,
                    &mut payload_file,
                    &mut payload_exhausted,
                    &mut payload_bytes_embedded,
                    &mut encoder,
                    &mut octx,
                )?;
            }
        }
    }
    //flush decoder
    decoder.send_eof()?;
    let mut frame = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut frame).is_ok() {
        process_frame(
            &mut frame,
            &mut payload_file,
            &mut payload_exhausted,
            &mut payload_bytes_embedded,
            &mut encoder,
            &mut octx,
        )?;
    }

    // Flush encoder
    encoder.send_eof()?;
    let mut packet = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut packet).is_ok() {
        packet.set_stream(0); //again this index is temp
        packet.write_interleaved(&mut octx)?;
    }

    octx.write_trailer()?;

    Ok(format!(
        "Successfully embedded {} bytes into output.mkv",
        payload_bytes_embedded
    ))
}

fn process_frame(
    frame: &mut ffmpeg::util::frame::video::Video,
    payload_file: &mut File,
    payload_exhausted: &mut bool,
    payload_bytes_embedded: &mut usize,
    encoder: &mut ffmpeg::encoder::video::Video,
    octx: &mut ffmpeg::format::context::Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel_count = frame.planes();

    // Only embed if payload isn't exhausted
    if !*payload_exhausted {
        for channel_index in 0..channel_count {
            let channel_data = frame.data_mut(channel_index);
            let capacity = (channel_data.len() - 32) / 8;
            let mut buffer: Vec<u8> = vec![0; capacity];
            let bytes_read = payload_file.read(&mut buffer)?;

            if bytes_read == 0 {
                *payload_exhausted = true;
                break;
            }

            *payload_bytes_embedded +=
                embed_image::embed_image(channel_data, &buffer[0..bytes_read], false)?;
        }
    }
    // Always encode the frame
    encoder.send_frame(frame)?;
    let mut packet = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut packet).is_ok() {
        packet.set_stream(0);
        packet.write_interleaved(octx)?;
    }

    Ok(())
}
