use ffmpeg_next::format::input;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::services::dct::dct_ii;
use crate::services::process_frame::{self, BufferGeneric};
use crate::services::qim::qim_extract;
use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub fn embed(object_path: PathBuf, configs: EmbedConfigs) -> Result<PathBuf, StegServiceError> {
    let output_payload = tempfile::NamedTempFile::new().map_err(|_| StegServiceError::FileError)?;
    let file_pointer =
        File::create(output_payload.path()).map_err(|_| StegServiceError::FileError)?;
    let mut buffer = BufferGeneric {
        reader: None,
        writer: Some(BufWriter::new(file_pointer)),
        buffer: [0; 1028],
        bit_index: 0,
        bits_read: 0,
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

    todo!()
}

fn drain_decoer(
    decoder: &mut ffmpeg_next::codec::decoder::Video,
    configs: &EmbedConfigs,
    buffer: &mut BufferGeneric,
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
    buffer: &mut BufferGeneric,
    plane_id: usize,
    plane_width: u32,
    plane_height: u32,
) -> Result<(), StegServiceError> {
    let stride = frame.stride(plane_id);
    let frame_data = frame.data_mut(plane_id);
    let mut payload_full = false;

    for row in 0..(plane_height / 4) {
        if payload_full {
            break;
        }
        for col in 0..(plane_width / 4) {
            if payload_full {
                break;
            }
            let frame_data_index = (row * 4) as usize * stride + (col * 4) as usize;

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
                let payload_bit = qim_extract(frame_dct_representation, configs.delta, i);
            }
        }
    }

    todo!()
}
