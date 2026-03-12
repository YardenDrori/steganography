use crate::services::process_frame::BLOCKS_PER_MACROBLOCK;
use crate::services::stdm;
use ffmpeg_next::format::input;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::services::dct::{dct_ii, idct_ii};
use crate::services::process_frame::{self};
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

const HEADER_SIZE_BITS: usize = 64;

struct PayloadBuffer {
    pub reader: BufReader<File>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
    pub bits_read: usize,
}

struct PendingBlock {
    coeffs: [f64; 16],
    coeffs_left_to_embed: usize,
}

struct EmbedState {
    pub coeffs_to_embed_count_block: usize,
    pub payload_exhausted: bool,
    pub pending_blocks: HashMap<u32, PendingBlock>,
    pub coeff_accumulator_pos: Vec<(u32, u8)>,
}

fn get_coeff(state: *const EmbedState, id: usize) -> Result<f64, StegServiceError> {
    let state = unsafe { &*state };

    let (block_offset, coeff_index) = state.coeff_accumulator_pos[id];
    let coeff = state
        .pending_blocks
        .get(&block_offset)
        .ok_or(StegServiceError::HashMapCallWithInvalidKey)?
        .coeffs[coeff_index as usize];

    Ok(coeff)
}

fn set_coeff(state: *mut EmbedState, id: usize, new_value: f64) -> Result<(), StegServiceError> {
    let state = unsafe { &mut *state };

    let (block_offset, coeff_index) = state.coeff_accumulator_pos[id];

    let block = state
        .pending_blocks
        .get_mut(&block_offset)
        .ok_or(StegServiceError::Other(
            "called hashmap with invalid key".to_string(),
        ))?;

    block.coeffs[coeff_index as usize] = new_value;

    Ok(())
}

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
        coeffs_to_embed_count_block: {
            let mut sum = 0;
            for i in 0..16 {
                if configs.coefficients_to_embed[i] {
                    sum += 1;
                }
            }
            sum
        },
        payload_exhausted: false,
        pending_blocks: HashMap::new(), //no hint here cause its annoying to calculate for little value
        coeff_accumulator_pos: Vec::with_capacity(configs.coefficients_per_bit),
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
                &mut service_state,
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
        &mut service_state,
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
    state: &mut EmbedState,
) -> Result<(), StegServiceError> {
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                process_frame::process_frame(configs, state, buffer, &mut frame, embed_in_channel)?;
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
    configs: &EmbedConfigs,
    state: &mut EmbedState,
    payload_buffer: &mut PayloadBuffer,
    frame: &mut ffmpeg_next::frame::Video,
    plane_height: u32,
    plane_width: u32,
    plane_id: usize,
) -> Result<(), StegServiceError> {
    if !frame.is_key() {
        return Ok(());
    }

    //clear any leftovers from previous uncompleted channel embed attempts
    state.coeff_accumulator_pos.clear();
    state.pending_blocks.clear();

    let stride = frame.stride(plane_id);
    let frame_data = frame.data_mut(plane_id);

    for block_row in 0..plane_height / 4 / BLOCKS_PER_MACROBLOCK {
        if state.payload_exhausted {
            break;
        }

        for block_col in 0..plane_width / 4 / BLOCKS_PER_MACROBLOCK {
            if state.payload_exhausted {
                break;
            }

            //we do this fancy math to translate the block coordinate to memory coordinates in the
            //frame.data() array while taking into consideration stride additionally we can change
            //the paramater BLOCKS_PER_MACROBLOCK to 1,2,4 to change step size from 4x4 blocks, 8x8
            //blocks to 16x16 aka a macro block for best naive robustness
            let block_offset = 4 * block_row * stride as u32 * BLOCKS_PER_MACROBLOCK
                + 4 * block_col * BLOCKS_PER_MACROBLOCK;

            let mut block_as_pixel = [0u8; 16];
            for i in 0..4 {
                for j in 0..4 {
                    block_as_pixel[i * 4 + j] = frame_data[block_offset as usize + i * stride + j];
                }
            }
            let block_as_dct = dct_ii(&block_as_pixel);

            state.pending_blocks.insert(
                block_offset,
                PendingBlock {
                    coeffs: block_as_dct,
                    coeffs_left_to_embed: state.coeffs_to_embed_count_block,
                },
            );

            for i in 0..16 {
                if !configs.coefficients_to_embed[i] {
                    continue;
                }

                state.coeff_accumulator_pos.push((block_offset, i as u8));
                state
                    .pending_blocks
                    .get_mut(&block_offset)
                    .ok_or(StegServiceError::HashMapCallWithInvalidKey)?
                    .coeffs_left_to_embed -= 1;

                if state.coeff_accumulator_pos.len() >= configs.coefficients_per_bit {
                    embed_bit_in_coefficients(state, payload_buffer, configs)?;

                    state.pending_blocks.retain(|offset, block| {
                        if block.coeffs_left_to_embed == 0 {
                            apply_modified_dct_coeffs_on_frame(
                                &block.coeffs,
                                frame_data,
                                *offset,
                                stride,
                            );
                            return false;
                        }
                        return true;
                    });

                    state.coeff_accumulator_pos.clear();
                }
            }
        }
    }

    // flush remaining pending blocks back to frame
    state.pending_blocks.retain(|offset, block| {
        apply_modified_dct_coeffs_on_frame(&block.coeffs, frame_data, *offset, stride);
        return false;
    });

    Ok(())
}

fn embed_bit_in_coefficients(
    state: &mut EmbedState,
    payload_buffer: &mut PayloadBuffer,
    configs: &EmbedConfigs,
) -> Result<(), StegServiceError> {
    if state.payload_exhausted {
        return Ok(());
    }
    let target_bit: bool;
    if payload_buffer.bit_index >= payload_buffer.bits_read {
        populate_payload_buffer(state, payload_buffer)?;
        if state.payload_exhausted {
            return Ok(());
        }
    }

    let target_byte = payload_buffer.buffer[payload_buffer.bit_index / 8];
    target_bit = (target_byte >> (7 - (payload_buffer.bit_index % 8))) & 0x1 == 0x1; //MSB
    payload_buffer.bit_index += 1;

    //We do unsafe here as this is the only way to avoid either coppying the entire coefficients to
    //a local variable and then having to coppy them back or exposing implementation details of the
    //embed method this way we just send two methods and this is safe as the steg thread is
    //blocking thus it is quite literally physically impossible for both get and set to be called
    //simultaneously
    let state_ptr = state as *mut EmbedState;
    stdm::stdm_embed(
        |i| get_coeff(state_ptr, i),
        |i, v| set_coeff(state_ptr, i, v),
        configs.coefficients_per_bit,
        configs.seed.clone(),
        target_bit,
        configs.delta,
    )?;

    Ok(())
}

fn apply_modified_dct_coeffs_on_frame(
    block_as_dct: &[f64; 16],
    data: &mut [u8],
    block_offset: u32,
    stride: usize,
) {
    let block_as_pixels = idct_ii(block_as_dct);

    for i in 0..4 {
        for j in 0..4 {
            data[i * stride + block_offset as usize + j] = block_as_pixels[i * 4 + j];
        }
    }
}

/// NOTE: assumes header bits are fully consumed before first call
fn populate_payload_buffer(
    embed_state: &mut EmbedState,
    buffer: &mut PayloadBuffer,
) -> Result<(), StegServiceError> {
    if buffer.bits_read == 0 {
        embed_state.payload_exhausted = true;
        return Ok(());
    }

    buffer.bits_read = 0;
    buffer.bit_index = 0;

    let buffer_size_bytes = buffer.buffer.len();

    while buffer.bits_read < buffer_size_bytes * 8 {
        let bytes_read = buffer
            .reader
            .read(&mut buffer.buffer[(buffer.bits_read / 8)..buffer_size_bytes])
            .map_err(|_| StegServiceError::FileError)?;

        if bytes_read == 0 {
            break;
        }

        buffer.bits_read += bytes_read * 8;
    }

    if buffer.bits_read == 0 {
        embed_state.payload_exhausted = true;
    }

    Ok(())
}
