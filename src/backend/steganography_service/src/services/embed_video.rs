use ffmpeg_next::{format::input, threading::Config};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::services::dct::dct_ii;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

struct Buffer {
    reader: BufReader<File>,
    buffer: [u8; 1028],
    index: usize,
    bytes_read: usize,
}

pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    //prepare buffer for reading payload data
    // let bits_per_block = {
    //     let mut sum = 0;
    //     for i in 0..16 {
    //         if configs.coefficients_to_embed[i] {
    //             sum += 1;
    //         }
    //     }
    //     sum
    // };
    let file_pointer = File::open(payload_path).map_err(|_| StegServiceError::FileError)?;
    let mut buffer = Buffer {
        reader: BufReader::new(file_pointer),
        buffer: [0; 1028],
        index: 0,
        // we initialize this as 1 as we use this to know if we finished embedding by checking if
        // this value is ever 0 signifying we read through the entire payload
        bytes_read: 1,
    };

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
                &configs,
                &mut buffer,
            )?;
        }
    }

    // flush decoder
    decoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    //TODO: get bits from payload here and pass them along
    drain_decoder(
        &mut decoder,
        &mut encoder,
        &mut output_context,
        output_stream_index,
        input_time_base,
        output_time_base,
        &configs,
        &mut buffer,
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
    configs: &EmbedConfigs,
    buffer: &mut Buffer,
) -> Result<(), StegServiceError> {
    let mut all_data_embedded: bool = false;
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                let modified_frame = process_frame(&frame, &configs, buffer)?;
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
    frame: &ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    buffer: &mut Buffer,
) -> Result<ffmpeg_next::frame::Video, StegServiceError> {
    const Y_PLANE: usize = 0;
    const CB_PLANE: usize = 1;
    const CR_PLANE: usize = 2;

    //allows to add more codecs, eng RGB also allows prioritizing best channel to embed in by
    //changing order of if statements for sensible defaults
    if let Some(yuv) = &configs.channels_to_embed.yuv {
        if yuv.y {
            embed_in_channel(frame, configs, buffer, Y_PLANE)?;
        }
        if yuv.cb {
            embed_in_channel(frame, configs, buffer, CB_PLANE)?;
        }
        if yuv.cr {
            embed_in_channel(frame, configs, buffer, CR_PLANE)?;
        }
    }

    todo!()
}

fn embed_in_channel(
    frame: &ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    buffer: &mut Buffer,
    plane_id: usize,
) -> Result<(), StegServiceError> {
    //buffer chenanigans
    if buffer.bytes_read != 0 && buffer.index >= buffer.bytes_read {
        buffer.bytes_read = buffer
            .reader
            .read(&mut buffer.buffer)
            .map_err(|_| StegServiceError::FileError)?;
        buffer.index = 0;
    }

    let mut data = frame.data(plane_id);
    let mut data_index = 0;
    while data_index <= data.len() {
        //todo
    }

    todo!()
}
