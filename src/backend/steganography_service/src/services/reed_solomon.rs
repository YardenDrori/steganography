use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use tempfile::tempfile;

use crate::{
    dtos::EmbedConfigs,
    errors::steg_service_error::StegServiceError,
    services::{gause_field::poly_div_remainder_vecs, rs_generator_vec::generatae_generator},
};

pub fn reed_solomon_encode(
    payload_path: PathBuf,
    configs: &EmbedConfigs,
) -> Result<(), StegServiceError> {
    let output_path = format!("{}_encoded", &payload_path.display());

    let input_file_pointer = File::open(&payload_path).map_err(|_| StegServiceError::FileError)?;
    let output_file_pointer =
        File::create(&output_path).map_err(|_| StegServiceError::FileError)?;

    let mut payload_bytes_left = input_file_pointer
        .metadata()
        .map_err(|_| StegServiceError::FileError)?
        .len() as i128;

    let mut reader = BufReader::new(input_file_pointer);
    let mut writer = BufWriter::new(output_file_pointer);

    let mut buffer: Vec<u8> = Vec::new();

    if configs.reed_solomon_padding_byte_count > 254 {
        return Err(StegServiceError::InvalidPayload);
    }

    let payload_bytes_per_chunk = 255 - configs.reed_solomon_padding_byte_count;

    let generator = generatae_generator(configs.reed_solomon_padding_byte_count)?;
    while payload_bytes_left > 0 {
        if payload_bytes_left > payload_bytes_per_chunk as i128 {
            buffer = vec![0; payload_bytes_per_chunk as usize];
        } else {
            buffer = vec![0; payload_bytes_left as usize];
        }

        reader
            .read_exact(&mut buffer)
            .map_err(|_| StegServiceError::FileError)?;

        encode_chunk(&mut buffer, &generator)?;

        writer
            .write_all(&buffer)
            .map_err(|_| StegServiceError::FileError)?;

        payload_bytes_left -= payload_bytes_per_chunk as i128;
    }

    writer.flush().map_err(|_| StegServiceError::FileError)?;

    std::fs::remove_file(&payload_path).map_err(|_| StegServiceError::FileError)?;
    std::fs::rename(output_path, payload_path).map_err(|_| StegServiceError::FileError)?;
    Ok(())
}

pub fn reed_solomon_decode() {
    todo!()
}

pub fn encode_chunk(chunk: &mut Vec<u8>, generator: &[u8]) -> Result<(), StegServiceError> {
    let chunk_len = chunk.len();

    //not 256 cause we cant use 0 as LOG_TABLE[0] is undefined
    if generator.len() - 1 + chunk_len > 255 {
        return Err(StegServiceError::ReedSolomonError(
            "remainder and chunk size sum exceeds 255".to_string(),
        ));
    }

    chunk.append(&mut vec![0u8; generator.len() - 1]);

    let remainder = poly_div_remainder_vecs(&chunk, generator);

    for i in 0..remainder.len() {
        chunk[chunk_len + i] = remainder[i];
    }

    Ok(())
}
