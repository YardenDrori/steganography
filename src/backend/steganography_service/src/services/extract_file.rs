use crate::services::embed_video::HEADER_SIZE_BITS;
use crate::services::process_frame::BLOCKS_PER_MACROBLOCK;
use ffmpeg_next::format::input;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use crate::services::dct::dct_ii;
use crate::services::process_frame::{self};
use crate::services::qim::qim_extract;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

struct PayloadBuffer {
    pub writer: BufWriter<File>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
}

struct ExtractState {
    payload_size: u64,
    coeff_accumulator: Vec<f64>,
    total_extracted_bytes: u64,
    extraction_ongoing: bool,
}

pub fn extract(object_path: PathBuf, configs: EmbedConfigs) -> Result<PathBuf, StegServiceError> {
    // ======== BUFFER & STATE SETUP ========
    //create the extracted payload file and assign it to the buffer
    let output_payload = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    let file_pointer =
        File::create(output_payload.path()).map_err(|_| StegServiceError::FileError)?;
    let mut buffer = PayloadBuffer {
        writer: BufWriter::new(file_pointer),
        buffer: [0; 1028],
        bit_index: 0,
    };

    //state setup
    let mut service_state = ExtractState {
        //0 here means unknown
        payload_size: 0,
        coeff_accumulator: Vec::with_capacity(configs.coefficients_per_bit),
        total_extracted_bytes: 0,
        extraction_ongoing: true,
    };

    // ======== FFMPEG I/O SETUP  ========
    ffmpeg_next::init().map_err(|e| StegServiceError::FfmpegError(e))?;

    //gathering data on the steg_obj file
    let mut input_context = input(&object_path).map_err(|_| StegServiceError::FileError)?;

    //getting the Video stream of the steg_object file
    let input_stream = input_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::StreamNotFound,
        ))?;
    let input_index = input_stream.index();
    let input_params = input_stream.parameters();

    //setup the decoder and relevant paramaters
    //ffmpeg's Black box that takes in packets and outputs frames
    let mut decoder = ffmpeg_next::codec::Context::from_parameters(input_params)
        .map_err(|e| StegServiceError::FfmpegError(e))?
        .decoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    // ======== PROCESSING LOOP ========
    for (stream, packet) in input_context.packets() {
        if stream.index() == input_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| StegServiceError::FfmpegError(e))?;

            drain_decoer(&mut decoder, &configs, &mut buffer, &mut service_state)?;
        }
    }

    // ======== CLEAN UP ========
    // flush decoder
    decoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    drain_decoer(&mut decoder, &configs, &mut buffer, &mut service_state)?;

    // flush any remaining partial buffer (last chunk may not be exactly 1028 bytes)
    if buffer.bit_index > 0 {
        let bytes_to_write = (buffer.bit_index + 7) / 8;
        buffer
            .writer
            .write_all(&buffer.buffer[..bytes_to_write])
            .map_err(|_| StegServiceError::FileError)?;
    }
    buffer
        .writer
        .flush()
        .map_err(|_| StegServiceError::FileError)?;
    drop(buffer.writer);

    // the embedder wrote an 8-byte LE u64 header first; read it to get the payload size,
    // then copy exactly that many bytes to a clean output file
    let mut raw = File::open(output_payload.path()).map_err(|_| StegServiceError::FileError)?;
    let mut header = [0u8; 8];
    raw.read_exact(&mut header)
        .map_err(|_| StegServiceError::FileError)?;
    let payload_size = u64::from_le_bytes(header);

    //generate the final file the user will get
    let final_output = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    {
        let mut final_file =
            File::create(final_output.path()).map_err(|_| StegServiceError::FileError)?;
        std::io::copy(&mut raw.take(payload_size), &mut final_file)
            .map_err(|_| StegServiceError::FileError)?;
    }

    //make file not temp and get its path for the return param of this method
    let (_, output_path) = final_output
        .keep()
        .map_err(|_| StegServiceError::FileError)?;
    Ok(output_path)
}

fn drain_decoer(
    decoder: &mut ffmpeg_next::codec::decoder::Video,
    configs: &EmbedConfigs,
    buffer: &mut PayloadBuffer,
    state: &mut ExtractState,
) -> Result<(), StegServiceError> {
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                process_frame::process_frame(
                    configs,
                    state,
                    buffer,
                    &mut frame,
                    extract_from_channel,
                )?;
            }
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            Err(ffmpeg_next::Error::Other { errno: 11 }) => return Ok(()),
            Err(e) => return Err(StegServiceError::FfmpegError(e)),
        }
    }
}

fn extract_from_channel(
    configs: &EmbedConfigs,
    state: &mut ExtractState,
    payload_buffer: &mut PayloadBuffer,
    frame: &mut ffmpeg_next::frame::Video,
    plane_width: u32,
    plane_height: u32,
    plane_id: usize,
) -> Result<(), StegServiceError> {
    //only check I frames (for now)
    if !frame.is_key() {
        return Ok(());
    }

    let stride = frame.stride(plane_id);
    let frame_data = frame.data_mut(plane_id);

    for block_row in 0..(plane_height / 4 / BLOCKS_PER_MACROBLOCK as u32) {
        for block_col in 0..(plane_width / 4 / BLOCKS_PER_MACROBLOCK as u32) {
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

            for i in 0..16 {
                if !configs.coefficients_to_embed[i] {
                    continue;
                }
                state.coeff_accumulator.push(block_as_dct[i]);
                if state.coeff_accumulator.len() >= configs.coefficients_per_bit {
                    let extracted_bit = extract_bit_from_coefficients()?;
                    write_bit_to_payload_buffer(extracted_bit, payload_buffer, state, configs)?;
                    state.coeff_accumulator.clear();
                }
            }
        }
    }
    Ok(())
}

fn extract_bit_from_coefficients() -> Result<bool, StegServiceError> {
    todo!()
}

fn write_bit_to_payload_buffer(
    target_bit: bool,
    buffer: &mut PayloadBuffer,
    state: &mut ExtractState,
) -> Result<(), StegServiceError> {
    if !state.extraction_ongoing {
        return Ok(());
    }

    // MSB
    buffer.buffer[buffer.bit_index / 8] <<= 1;
    if target_bit {
        buffer.buffer[buffer.bit_index / 8] |= 0x1;
    }
    buffer.bit_index += 1;

    // one time header parse, same moment every run, no special casing
    if state.payload_size == 0 && buffer.bit_index == HEADER_SIZE_BITS {
        state.payload_size = u64::from_le_bytes(
            buffer.buffer[0..8]
                .try_into()
                .map_err(|_| StegServiceError::FileError)?,
        );
        buffer.bit_index = 0;
        buffer.buffer = [0; 1028];
        state.total_extracted_bytes = 0;
        return Ok(());
    }

    // flush buffer to file when full
    if buffer.bit_index >= buffer.buffer.len() * 8 {
        buffer
            .writer
            .write_all(&buffer.buffer[0..buffer.bit_index / 8])
            .map_err(|_| StegServiceError::FileError)?;
        state.total_extracted_bytes += buffer.bit_index as u64 / 8;
        if state.total_extracted_bytes >= state.payload_size {
            state.extraction_ongoing = false;
        }
        buffer.bit_index = 0;
        buffer.buffer = [0; 1028];
    }

    Ok(())
}
