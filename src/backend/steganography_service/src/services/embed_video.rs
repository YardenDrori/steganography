use crate::services::process_frame::BLOCKS_PER_MACROBLOCK;
use crate::services::stdm;
use ffmpeg_next::format::input;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::services::dct::{dct_ii, idct_ii};
use crate::services::process_frame::{self};
use crate::services::qim::qim_embed;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

// the method i hate the most as i feel it is quite messy and unorganized
pub struct PayloadBuffer {
    pub reader: BufReader<File>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
    pub bits_read: usize,
}

struct EmbedState {
    pub payload_exhausted: bool,
    pub coeffs_for_bit_index: u8,
}

const HEADER_SIZE_BITS: usize = 64;
pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    // ======== PAYLOAD & STATE SETUP ========
    let file_pointer = File::open(payload_path).map_err(|_| StegServiceError::FileError)?;
    let file_size = file_pointer
        .metadata()
        .map_err(|_| StegServiceError::FileError)?
        .len()
        .to_le_bytes();

    //buffer setup
    let mut buffer = PayloadBuffer {
        reader: BufReader::new(file_pointer),
        buffer: [0; 1028],
        bit_index: 0,
        // we initialize this as 64 becasue we use this the first 64 bits to embed the header which
        // tells the extractor how many bits are there in this file
        bits_read: HEADER_SIZE_BITS,
    };
    buffer.buffer[0..8].copy_from_slice(&file_size);

    //state setup
    let mut service_state = EmbedState {
        payload_exhausted: false,
        coeffs_for_bit_index: 0,
    };

    // ======== FFMPEG I/O SETUP (the big chonker) ========
    ffmpeg_next::init().map_err(|e| StegServiceError::FfmpegError(e))?;

    //gathering data on the carrier file
    let mut input_context = input(&carrier_path).map_err(|_| StegServiceError::FileError)?;
    let input_format_name = input_context
        .format()
        .name()
        .split(',')
        .next()
        .unwrap_or("mp4")
        .to_string();

    //gathering data on the video stream of the carrier
    let input_stream = input_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::StreamNotFound,
        ))?;
    let input_index = input_stream.index();
    let input_params = input_stream.parameters();
    let input_time_base = input_stream.time_base();

    //setup the decoder and relevant paramaters
    //ffmpeg's Black box that takes in packets and outputs frames
    let mut decoder = ffmpeg_next::codec::Context::from_parameters(input_params.clone())
        .map_err(|e| StegServiceError::FfmpegError(e))?
        .decoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    let input_codec = decoder.codec().ok_or(StegServiceError::FfmpegError(
        ffmpeg_next::Error::InvalidData,
    ))?;

    //prepare info for the steg_object
    let output_path = PathBuf::from(format!(
        "embedded_{}",
        carrier_path.file_name().unwrap().to_string_lossy()
    ));

    //prep output context which knows a buncha stuff on how the output file will be configured
    let mut output_context = ffmpeg_next::format::output_as(&output_path, &input_format_name)
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    //Prep the encoder with the same codec of the carrier
    //ffmpeg's black box that takes in frames and outputs packets
    let mut encoder = ffmpeg_next::codec::Context::new_with_codec(input_codec)
        .encoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    //configure encoder to use the same paramaters as the input carrier
    encoder.set_time_base(input_time_base);
    encoder
        .set_parameters(input_params.clone())
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    //write a global header in the file so that SPS and PPS only get stored once in the file header
    //otherwise they get stored in every frame
    unsafe {
        let oformat = (*output_context.as_ptr()).oformat;
        //null check and ensure that oformat has the global header flag via bit twiddling
        if !oformat.is_null() && (*oformat).flags & ffmpeg_sys_next::AVFMT_GLOBALHEADER as i32 != 0
        {
            (*encoder.as_mut_ptr()).flags |= ffmpeg_sys_next::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }
    }

    //Finalize the encoder (does a buncha shit behind the scenes like generating SPS and PPS which
    //are fancy instructions on how to read the file)
    let mut encoder = encoder
        .open()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    //add the video stream to the output_context,
    //populate the codecpar which is just a struct that lives in the file alognside the stream
    //holding information
    let output_stream_index = {
        let mut stream = output_context
            .add_stream(encoder.codec())
            .map_err(|e| StegServiceError::FfmpegError(e))?;
        unsafe {
            //take the paramaters from the encoder which generated them
            let res = ffmpeg_sys_next::avcodec_parameters_from_context(
                (*stream.as_mut_ptr()).codecpar,
                encoder.as_ptr(),
            );
            if res < 0 {
                return Err(StegServiceError::FfmpegError(ffmpeg_next::Error::from(res)));
            }
        }
        stream.index()
    };

    //write the header from memory to disk, we need to do this now as the time base isnt finalized
    //and basically is in a quantum superposition untill we do this
    output_context
        .write_header()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    //get the output's time base this needs to be after we write the header because... reasons
    let output_time_base = output_context
        .stream(output_stream_index)
        .unwrap()
        .time_base();

    // ======== PROCESSING LOOP ========
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

    //if the payload isnt exhausted we didnt write the entire payload but did finish the loop
    //indicating that the carrier did not have sufficient capacity for the payload
    if !service_state.payload_exhausted {
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

    //write to disk the finalized embedded file
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
    buffer: &mut PayloadBuffer,
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
            //error 11 is unable to generate frames due to missing info indicating more packets
            //need to be fed to the decoder which is expected behavior
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

                //converts the packet from the source time base to the output time base as time
                //bases are so sensitive literally anything could make them change
                packet.rescale_ts(input_time_base, output_time_base);

                packet
                    .write_interleaved(output_context)
                    .map_err(|e| StegServiceError::FfmpegError(e))?;
            }
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            //error 11 is unable to generate packets due to missing info indicating more frames
            //need to be fed to the encoder which is expected behavior
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

            //like seriously the nest depth here comes to a max of 6 deep jesus
            for i in 0..16 {
                if configs.coefficients_to_embed[i] == false {
                    continue;
                }

                //progress payload - i know u said dont explain what explain why but honestly this
                //code is so messy im struggling to follow otherwise
                if payload_buffer.coeffs_for_bit_index >= configs.coefficients_per_bit {
                    //hell yeah i love handling like 3 indeces at once
                    payload_buffer.bit_index += 1;
                    payload_buffer.coeffs_for_bit_index = 0;

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
                } else {
                    payload_buffer.coeffs_for_bit_index += 1;
                }

                // hell yeah i LOVE bit twiddling 😭😭😭
                // yeah i know we can cache this as we calc this multiple times per bit im too
                // spitefull of the code to do so
                let target_byte = payload_buffer.buffer[payload_buffer.bit_index / 8];
                let target_bit = (target_byte >> 7 - payload_buffer.bit_index % 8) & 0x1 == 0x1;

                let dc_before = frame_dct_representation[crate::services::qim::ZIGZAG[i]];

                stdm::stdm_embed(
                    &mut frame_dct_representation,
                    &configs.seed,
                    target_bit,
                    configs.delta,
                )?;

                let dc_after = frame_dct_representation[crate::services::qim::ZIGZAG[i]];

                //logging
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
