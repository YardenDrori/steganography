use crate::extract_image;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use ffmpeg_next::{self as ffmpeg};

pub fn extract_video(
    steg_file_location: &str,
    extracted_payload_location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    println!("\n=== Extracting Payload ===");
    println!("Steg file: {}",
        steg_file_location.split('/').last().unwrap_or(steg_file_location));

    let mut output_file = BufWriter::new(File::create(extracted_payload_location)?);
    let mut bytes_extracted = 0;

    // === DECODING ===
    //input context - handle for the video
    let mut ictx = ffmpeg::format::input(steg_file_location)?;
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
                process_frame(&mut frame, &mut output_file, &mut payload_exhausted, &mut bytes_extracted)?;
            }
        }
    }
    //flush decoder
    decoder.send_eof()?;
    let mut frame = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut frame).is_ok() {
        process_frame(&mut frame, &mut output_file, &mut payload_exhausted, &mut bytes_extracted)?;
    }

    output_file.flush()?;

    println!("\n=== Extraction Complete ===");
    println!("Extracted: {:.2} KB", bytes_extracted as f64 / 1024.0);
    println!("Output: {}", extracted_payload_location.split('/').last().unwrap_or(extracted_payload_location));

    Ok(format!("Successfully extracted {} bytes", bytes_extracted))
}

fn process_frame(
    frame: &mut ffmpeg::util::frame::video::Video,
    output_file: &mut BufWriter<File>,
    payload_exhausted: &mut bool,
    bytes_extracted: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel_count = frame.planes();
    for channel_index in 0..channel_count {
        if *payload_exhausted {
            break;
        }

        let channel_data = frame.data_mut(channel_index);
        let buffer = extract_image::extract_from_image(channel_data)?;
        match buffer {
            Some(buffer) => {
                *bytes_extracted += buffer.len();
                output_file.write_all(&buffer)?;
            }
            None => {
                *payload_exhausted = true;
                break;
            }
        }
    }

    Ok(())
}
