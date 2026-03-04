use ffmpeg_next::codec::traits::Encoder;
use ffmpeg_next::codec::{Parameters, codec};
use ffmpeg_next::format::input;
use ffmpeg_next::{decoder::Video, encoder};
use std::path::PathBuf;

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub fn embed(
    payload_path: PathBuf,
    carrier_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<PathBuf, StegServiceError> {
    ffmpeg_next::init().map_err(|e| StegServiceError::FfmpegError(e))?;

    let mut input_context = input(&carrier_path).map_err(|_| StegServiceError::FileError)?;
    let input = input_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::StreamNotFound,
        ))?;

    let input_index = input.index();
    let input_params = input.parameters();
    let codec_context = ffmpeg_next::codec::Context::from_parameters(input_params.clone())
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    let mut decoder = codec_context
        .decoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    let input_codec = decoder
        .codec()
        .ok_or(StegServiceError::FfmpegError(
            ffmpeg_next::Error::InvalidData,
        ))?
        .id();

    //finally we cast start doing shit with frames jesus christ ffmpeg has so much setup to it
    for (stream, packet) in input_context.packets() {
        if stream.index() == input_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| StegServiceError::FfmpegError(e))?;

            extract_and_process_frame(&mut decoder)?;
        }
    }
    decoder
        .send_eof()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    extract_and_process_frame(&mut decoder)?;

    let codec = ffmpeg_next::codec::encoder::find(input_codec).ok_or(
        StegServiceError::FfmpegError(ffmpeg_next::Error::EncoderNotFound),
    )?;
    let encoder_context = ffmpeg_next::codec::Context::new_with_codec(codec);
    let mut encoder = encoder_context
        .encoder()
        .video()
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    encoder
        .set_parameters(input_params)
        .map_err(|e| StegServiceError::FfmpegError(e))?;
    encoder
        .open()
        .map_err(|e| StegServiceError::FfmpegError(e))?;

    todo!()
}

fn extract_and_process_frame(decoder: &mut Video) -> Result<(), StegServiceError> {
    loop {
        let mut frame = ffmpeg_next::frame::Video::empty();
        match decoder.receive_frame(&mut frame) {
            Ok(_) => process_frame(&decoder),
            Err(ffmpeg_next::Error::Eof) => return Ok(()),
            Err(ffmpeg_next::Error::Other { errno: 11 }) => continue,
            Err(e) => return Err(StegServiceError::FfmpegError(e)),
        }
    }
}

fn process_frame(mut _decoder: &Video) {
    todo!()
}

//#[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;
//
//     #[test]
//     fn test_add() {
//     }
//
// }
