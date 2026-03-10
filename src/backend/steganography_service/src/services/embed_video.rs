use crate::services::process_frame::BLOCKS_PER_MACROBLOCK;
use ffmpeg_next::format::input;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::services::dct::{dct_ii, idct_ii};
use crate::services::process_frame::{self};
use crate::services::qim::qim_embed;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub struct Buffer {
    pub reader: BufReader<File>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
    pub bits_read: usize,
    pub payload_exhausted: bool,
    pub blocks_logged: usize,
}

pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    //prepare buffer for reading payload data
    // let bits_per_block: u8 = {
    //     let mut sum = 0;
    //     for i in 0..16 {
    //         if configs.coefficients_to_embed[i] {
    //             sum += 1;
    //         }
    //     }
    //     sum
    // };
    let file_pointer = File::open(payload_path).map_err(|_| StegServiceError::FileError)?;
    let file_size = file_pointer
        .metadata()
        .map_err(|_| StegServiceError::FileError)?
        .len()
        .to_le_bytes();
    let mut buffer = Buffer {
        reader: BufReader::new(file_pointer),
        buffer: [0; 1028],
        bit_index: 0,
        // we initialize this as 64 becasue we use this the first 64 bits to embed the header which
        // tells the extractor how many bits are there in this file
        bits_read: 64,
        payload_exhausted: false,
        blocks_logged: 0,
    };
    buffer.buffer[0..8].copy_from_slice(&file_size);

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

    let input_format_name = input_context
        .format()
        .name()
        .split(',')
        .next()
        .unwrap_or("mp4")
        .to_string();

    let input_codec = decoder
        .codec()
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::InvalidData,
        ))?
        .id();

    // --- OUTPUT (before encoder, so we can read the format's flags) ---
    let output_path = PathBuf::from(format!(
        "embedded_{}",
        carrier_path.file_name().unwrap().to_string_lossy()
    ));
    let mut output_context = ffmpeg_next::format::output_as(&output_path, &input_format_name)
        .map_err(|e| StegServiceError::FfmpegError(e))?;

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
    encoder.set_time_base(input_time_base);
    // MP4/MOV containers store H.264 SPS/PPS in the avcC box in the file
    // header rather than inline in every IDR packet. AVFMT_GLOBALHEADER tells
    // us the muxer requires this, and AV_CODEC_FLAG_GLOBAL_HEADER tells the
    // encoder to produce SPS/PPS as extradata instead of Annex-B start codes.
    // Without this the container header and the bitstream carry mismatched
    // SPS/PPS, which causes decoders to produce black or no video.
    unsafe {
        let oformat = (*output_context.as_ptr()).oformat;
        if !oformat.is_null() && (*oformat).flags & ffmpeg_sys_next::AVFMT_GLOBALHEADER as i32 != 0
        {
            (*encoder.as_mut_ptr()).flags |= ffmpeg_sys_next::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }
    }
    let mut encoder = encoder
        .open()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    let output_stream_index = {
        let mut stream = output_context
            .add_stream(encoder.codec())
            .map_err(|e| StegServiceError::FfmpegError(e))?;
        // Copy parameters from the opened encoder, not from the input stream.
        // The encoder's extradata now contains the freshly generated SPS/PPS
        // that must match what the muxer writes into the avcC box.
        unsafe {
            let ret = ffmpeg_sys_next::avcodec_parameters_from_context(
                (*stream.as_mut_ptr()).codecpar,
                encoder.as_ptr(),
            );
            if ret < 0 {
                return Err(StegServiceError::FfmpegError(ffmpeg_next::Error::from(ret)));
            }
        }
        stream.index()
    };
    output_context
        .write_header()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    // Read output_time_base AFTER write_header — the muxer finalises the
    // stream's time_base during write_header, so reading it beforehand gives
    // a stale/zero value that makes rescale_ts produce AV_NOPTS_VALUE.
    let output_time_base = output_context
        .stream(output_stream_index)
        .unwrap()
        .time_base();

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

    if !buffer.payload_exhausted {
        return Err(StegServiceError::InsufficientCapacity);
    }

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
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                process_frame::process_frame(&mut frame, configs, buffer, embed_in_channel)?;
                encoder
                    .send_frame(&frame)
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

fn embed_in_channel(
    frame: &mut ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    payload_buffer: &mut Buffer,
    plane_id: usize,
    plane_width: u32,
    plane_height: u32,
) -> Result<(), StegServiceError> {
    if !frame.is_key() {
        return Ok(());
    }
    let stride = frame.stride(plane_id);
    let frame_data = frame.data_mut(plane_id);

    for row in 0..(plane_height / 4 / BLOCKS_PER_MACROBLOCK as u32) {
        if payload_buffer.payload_exhausted {
            break;
        }
        for col in 0..(plane_width / 4 / BLOCKS_PER_MACROBLOCK as u32) {
            if payload_buffer.payload_exhausted {
                break;
            }

            let frame_data_index =
                (row as usize * stride * 4 + col as usize * 4) * BLOCKS_PER_MACROBLOCK as usize;

            //we extract the block and because we need a matrix slice of the array we gotta convert it
            //to a matrix (we still store it as an array that represents a matrix as i dont wanna
            //refactor dct and qim it makes no difference)
            let mut block = [0u8; 16];
            for i in 0..4 {
                for j in 0..4 {
                    block[j + i * 4] = frame_data[j + frame_data_index + i * stride];
                }
            }
            let mut frame_dct_representation = dct_ii(&block);

            for i in 0..16 {
                if configs.coefficients_to_embed[i] == false {
                    continue;
                }
                if payload_buffer.bit_index >= payload_buffer.bits_read {
                    payload_buffer.bits_read = payload_buffer
                        .reader
                        .read(&mut payload_buffer.buffer)
                        .map_err(|_| StegServiceError::FileError)?
                        * 8;
                    payload_buffer.bit_index = 0;
                    if payload_buffer.bits_read == 0 {
                        payload_buffer.payload_exhausted = true;
                        break;
                    }
                }
                // hell yeah i LOVE bit twiddling 😭😭😭
                let target_byte = payload_buffer.buffer[payload_buffer.bit_index / 8];
                let target_bit = (target_byte >> 7 - payload_buffer.bit_index % 8) & 0x1 == 0x1;

                let dc_before = frame_dct_representation[crate::services::qim::ZIGZAG[i]];

                payload_buffer.bit_index += 1;

                frame_dct_representation =
                    qim_embed(frame_dct_representation, target_bit, configs.delta, i);

                let dc_after = frame_dct_representation[crate::services::qim::ZIGZAG[i]];

                if payload_buffer.blocks_logged < 10 {
                    tracing::debug!(
                        "[EMBED] plane={} block=({},{}) coeff={} bit={} dc_before={:.1} dc_after={:.1}",
                        plane_id,
                        row,
                        col,
                        i,
                        target_bit as u8,
                        dc_before,
                        dc_after
                    );
                    payload_buffer.blocks_logged += 1;
                }
            }

            let embedded_block = idct_ii(&frame_dct_representation);
            for i in 0..4 {
                for j in 0..4 {
                    frame_data[j + frame_data_index + i * stride] = embedded_block[j + i * 4];
                }
            }
        }
    }
    Ok(())
}
