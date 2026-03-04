use ffmpeg_next::format::input;
use std::path::PathBuf;

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    _configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    ffmpeg_next::init().map_err(|e| StegServiceError::FfmpegError(e))?;

    // --- INPUT ---
    let mut input_context = input(&carrier_path).map_err(|_| StegServiceError::FileError)?;
    let input_stream = input_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::StreamNotFound,
        ))?;

    let input_index = input_stream.index();
    let input_params = input_stream.parameters();
    let input_time_base = input_stream.time_base();

    let mut decoder = ffmpeg_next::codec::Context::from_parameters(input_params.clone())
        .map_err(|e| StegServiceError::FfmpegError(e))?
        .decoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    let input_codec = decoder
        .codec()
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::InvalidData,
        ))?
        .id();

    // --- ENCODER ---
    let codec = ffmpeg_next::codec::encoder::find(input_codec).ok_or(
        StegServiceError::FfmpegError(ffmpeg_next::Error::EncoderNotFound),
    )?;
    let mut encoder = ffmpeg_next::codec::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    encoder
        .set_parameters(input_params.clone())
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    let mut encoder = encoder
        .open()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    // --- OUTPUT ---
    let output_path = PathBuf::from(format!(
        "embedded_{}",
        carrier_path.file_name().unwrap().to_string_lossy()
    ));
    let mut output_context =
        ffmpeg_next::format::output(&output_path).map_err(|e| StegServiceError::FfmpegError(e))?;
    let output_stream_index = {
        let mut stream = output_context
            .add_stream(encoder.codec())
            .map_err(|e| StegServiceError::FfmpegError(e))?;
        stream.set_parameters(input_params);
        stream.index()
    };
    let output_time_base = output_context
        .stream(output_stream_index)
        .unwrap()
        .time_base();
    output_context
        .write_header()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    // --- DECODE / PROCESS / ENCODE LOOP ---
    for (stream, packet) in input_context.packets() {
        if stream.index() == input_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| StegServiceError::FfmpegError(e))?;
            drain_decoder(
                &mut decoder,
                &mut encoder,
                &mut output_context,
                output_stream_index,
                input_time_base,
                output_time_base,
            )?;
        }
    }

    // flush decoder
    decoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    drain_decoder(
        &mut decoder,
        &mut encoder,
        &mut output_context,
        output_stream_index,
        input_time_base,
        output_time_base,
    )?;

    // flush encoder
    encoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    drain_encoder(
        &mut encoder,
        &mut output_context,
        output_stream_index,
        input_time_base,
        output_time_base,
    )?;

    output_context
        .write_trailer()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    Ok(output_path)
}

// receives frames from decoder, processes them, sends to encoder, drains encoder packets
fn drain_decoder(
    decoder: &mut ffmpeg_next::decoder::video::Video,
    encoder: &mut ffmpeg_next::encoder::Encoder,
    output_context: &mut ffmpeg_next::format::context::Output,
    output_stream_index: usize,
    input_time_base: ffmpeg_next::Rational,
    output_time_base: ffmpeg_next::Rational,
) -> Result<(), StegServiceError> {
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                let modified_frame = process_frame(&frame)?;
                encoder
                    .send_frame(&modified_frame)
                    .map_err(|e| StegServiceError::FfmpegError(e))?;
                drain_encoder(
                    encoder,
                    output_context,
                    output_stream_index,
                    input_time_base,
                    output_time_base,
                )?;
            }
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            Err(ffmpeg_next::Error::Other { errno: 11 }) => return Ok(()),
            Err(e) => return Err(StegServiceError::FfmpegError(e)),
        }
    }
}

// receives encoded packets from encoder and writes them to the output file
fn drain_encoder(
    encoder: &mut ffmpeg_next::encoder::Encoder,
    output_context: &mut ffmpeg_next::format::context::Output,
    output_stream_index: usize,
    input_time_base: ffmpeg_next::Rational,
    output_time_base: ffmpeg_next::Rational,
) -> Result<(), StegServiceError> {
    loop {
        let mut packet = ffmpeg_next::Packet::empty();
        match encoder.receive_packet(&mut packet) {
            Ok(_) => {
                packet.set_stream(output_stream_index);
                packet.rescale_ts(input_time_base, output_time_base);
                packet
                    .write_interleaved(output_context)
                    .map_err(|e| StegServiceError::FfmpegError(e))?;
            }
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            Err(ffmpeg_next::Error::Other { errno: 11 }) => return Ok(()),
            Err(e) => return Err(StegServiceError::FfmpegError(e)),
        }
    }
}

fn process_frame(
    _frame: &ffmpeg_next::frame::Video,
) -> Result<ffmpeg_next::frame::Video, StegServiceError> {
    todo!()
}
