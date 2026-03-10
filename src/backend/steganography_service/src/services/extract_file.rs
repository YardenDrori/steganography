use crate::services::process_frame::BLOCKS_PER_MACROBLOCK;
use ffmpeg_next::format::input;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use crate::services::dct::dct_ii;
use crate::services::process_frame::{self};
use crate::services::qim::qim_extract;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub struct Buffer {
    pub writer: BufWriter<File>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
    pub blocks_logged: usize,
}

pub fn extract(object_path: PathBuf, configs: EmbedConfigs) -> Result<PathBuf, StegServiceError> {
    let output_payload = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    let file_pointer =
        File::create(output_payload.path()).map_err(|_| StegServiceError::FileError)?;
    let mut buffer = Buffer {
        writer: BufWriter::new(file_pointer),
        buffer: [0; 1028],
        bit_index: 0,
        blocks_logged: 0,
    };

    ffmpeg_next::init().map_err(|e| StegServiceError::FfmpegError(e))?;

    let mut input_context = input(&object_path).map_err(|_| StegServiceError::FileError)?;
    let input_stream = input_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::StreamNotFound,
        ))?;

    let input_index = input_stream.index();
    let input_params = input_stream.parameters();

    let mut decoder = ffmpeg_next::codec::Context::from_parameters(input_params)
        .map_err(|e| StegServiceError::FfmpegError(e))?
        .decoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    for (stream, packet) in input_context.packets() {
        if stream.index() == input_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| StegServiceError::FfmpegError(e))?;

            drain_decoer(&mut decoder, &configs, &mut buffer)?;
        }
    }

    // flush decoder
    decoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    drain_decoer(&mut decoder, &configs, &mut buffer)?;

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

    let final_output = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    {
        let mut final_file =
            File::create(final_output.path()).map_err(|_| StegServiceError::FileError)?;
        std::io::copy(&mut raw.take(payload_size), &mut final_file)
            .map_err(|_| StegServiceError::FileError)?;
    }

    let (_, output_path) = final_output
        .keep()
        .map_err(|_| StegServiceError::FileError)?;
    Ok(output_path)
}

fn drain_decoer(
    decoder: &mut ffmpeg_next::codec::decoder::Video,
    configs: &EmbedConfigs,
    buffer: &mut Buffer,
) -> Result<(), StegServiceError> {
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => {
                process_frame::process_frame(&mut frame, configs, buffer, extract_from_channel)?;
            }
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            Err(ffmpeg_next::Error::Other { errno: 11 }) => return Ok(()),
            Err(e) => return Err(StegServiceError::FfmpegError(e)),
        }
    }
}

fn extract_from_channel(
    frame: &mut ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    buffer: &mut Buffer,
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
        for col in 0..(plane_width / 4 / BLOCKS_PER_MACROBLOCK as u32) {
            let frame_data_index =
                ((row as usize * stride * 4) + (col * 4) as usize) * BLOCKS_PER_MACROBLOCK as usize;

            //we extract the block and because we need a matrix slice of the array we gotta convert it
            //to a matrix (we still store it as an array that represents a matrix as i dont wanna
            //refactor dct and qim it makes no difference)
            let mut block = [0u8; 16];
            for i in 0..4 {
                for j in 0..4 {
                    block[j + i * 4] = frame_data[j + frame_data_index + i * stride];
                }
            }
            let frame_dct_representation = dct_ii(&block);

            for i in 0..16 {
                if configs.coefficients_to_embed[i] == false {
                    continue;
                }
                // hell yeah i LOVE bit twiddling (part 2) 😭😭😭
                let payload_bit = qim_extract(frame_dct_representation, configs.delta, i);

                if buffer.blocks_logged < 10 {
                    let coeff_val = frame_dct_representation[crate::services::qim::ZIGZAG[i]];
                    let delta_f = configs.delta as f64;
                    let dist_true = (coeff_val - (coeff_val / delta_f).round() * delta_f).abs();
                    let dist_false =
                        (coeff_val - ((coeff_val / delta_f - 0.5).round() + 0.5) * delta_f).abs();
                    tracing::debug!(
                        "[EXTRACT] plane={} block=({},{}) coeff={} dc={:.1} dist_true={:.1} dist_false={:.1} -> bit={}",
                        plane_id,
                        row,
                        col,
                        i,
                        coeff_val,
                        dist_true,
                        dist_false,
                        payload_bit as u8
                    );
                    buffer.blocks_logged += 1;
                }

                buffer.buffer[buffer.bit_index / 8] <<= 1;
                buffer.buffer[buffer.bit_index / 8] |= if payload_bit { 0x1 } else { 0x0 };
                buffer.bit_index += 1;
                if buffer.bit_index >= buffer.buffer.len() * 8 {
                    buffer
                        .writer
                        .write_all(&buffer.buffer)
                        .map_err(|_| StegServiceError::FileError)?;
                    buffer.bit_index = 0;
                    buffer.buffer = [0; 1028];
                }
            }
        }
    }
    Ok(())
}
