// the bible of reed solomon cause this algo is a bitch
// we take x bytes we wanna encode pad them with y parity bytes for every 2 parity bytes we can
// recover 1 error. now we use GF math here so plus and minus are both XOR we generate a table for
// quick multiplication and division via a generator (2) which is a number that can beacome any
// number in GF thanks to GF's special wrapping nature when a value goes abbove 255 (e.g
// multiplication) it wraps back in a special way eg we have 0b10000000 * 2 we do a shift left and
// if it overflows (which it does here) we do a xor with the GF version of a prime number (0x011D)
// after we generate the tables it gets much easier lets say we have EXP_TABLE so EXP_TABLE[0] =
// 2^0 EXP_TABLE[1] = 2^1 and so on note this is GF power of so we multiply with GF math the reason
// we need GF math is so that a byte will wrap in a usefull manner and that it will never have a
// decimal point anywho to multiply 4 with 8 for instance we take the position of 4 and eight in
// the exp table (via a log table) so for 4 its 2 and for 8 its 3 we add up their positions (with
// regular math here) which
// comes up to 5 and look and EXP_TABLE[5] for the result coming up to 32 (it is consistent
// with regular multiplication untill it wraps) for division we instead of adding up the positions
// we subtract them (again with regular math) if we overflow or underflow we add or subtract 255 (with regular math) untill we are between 0-255
// again
//
// to encode we use the generator element (2) to make a generator polynomal so if we wanted 5
// parity bytes the generator would be g(x) = (x-2^0)*(x-2^1)*(x-2^2)*(x-2^3)*(x-2^4) multiplying
// them will give us a formula that plugging any value from 2^0 to 2^4 makes g(x) = 0 this is
// usefull for for reasons that will be explained later, after we have the generator we take the
// original message pad a zero for each parity byte we said we wanted and divide that with the
// generator using long division we take whatever remainder is left which is gauaranteed to be at
// most as big as parity bytes because math is cool like that, we replace the zeros with the
// remainders this gives us the benifit of division with the generator now giving us zero which
// will be very usefull for decoding we are now done with the encoding step
//
// for decoding we take a vec of Syndromes aka how and where are the errors we find them by taking
// the message we received on our end and plugging j generator roots in it where j is how many
// parity bytes we added so for instance if the message on our end is m=1+2x+3x^2+4x^3 meaning first
// byte is 1 second is 2 third is 3 and so on and we had 2 parity bytes we plug 2^0 and 2^1 in the
// mplace of x which will produce 2 syndromes for us if all syndromes's value is 0 we're done this
// is why we wanted division to with the generator in earlier steps to equate 0 cause the error
// formula is something like this e(x) = message*x + error*x so now if we plug in one of the values
// from the generator roots e.g 2^0 e(1) = message*1 + error*1 thanks to how we built the message
// we know that regardless of what the mssage was it is now 0 leaving us with e(1) = error*1 which
// is very usefull for us we can expand error to be Y*X where Y is how off the byte is from
// what it was supposed to be and X is in which recieved byte the error is this doesnt tell us
// anything at all but these are the components of the error if we had more than 1 error the
// formula would be Y1*X1 Y2*X2 so we can make a polynom of all the errors called
// e(x) = Y1*X^1 + Y2*X^2 + ... Yn*X^n
// where n is how many errors we had during transmission this is why every 2 parity bytes can fix 1
// error because every error is made of amount and position and each parity byte will allow us to
// recover 1 of those for reasons that will be explained later
// in the encoding step we represented each byte's index with a power of x so we stick to that
// pattern here if the error is in byte 4 than the err is Y4*X^4
// the genric formula is
// e = Y0*X^0 + Y1*X^1 + ... Yi*X^i
//
//
//
//
//
//
//
//
//
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use crate::{
    dtos::EmbedConfigs,
    errors::steg_service_error::StegServiceError,
    services::{
        gause_field::{EXP_TABLE, poly_div_remainder_vecs, poly_mult},
        rs_generator_vec::{generatae_generator, get_roots_for_generator_with_len},
    },
};

pub fn reed_solomon_encode(
    payload_path: PathBuf,
    configs: EmbedConfigs,
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

    let mut buffer: Vec<u8>;

    if configs.reed_solomon_padding_byte_count > 254 {
        return Err(StegServiceError::InvalidPayload);
    }

    let payload_bytes_per_chunk = 255 - configs.reed_solomon_padding_byte_count;

    let generator = generatae_generator(configs.reed_solomon_padding_byte_count);
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

pub fn reed_solomon_decode(
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

    let mut buffer: Vec<u8>;

    if configs.reed_solomon_padding_byte_count > 254 {
        return Err(StegServiceError::InvalidPayload);
    }

    let payload_bytes_per_chunk = 255 - configs.reed_solomon_padding_byte_count;

    let generator_len = configs.reed_solomon_padding_byte_count;
    while payload_bytes_left > 0 {
        if payload_bytes_left > payload_bytes_per_chunk as i128 {
            buffer = vec![0; payload_bytes_per_chunk as usize];
        } else {
            buffer = vec![0; payload_bytes_left as usize];
        }

        reader
            .read_exact(&mut buffer)
            .map_err(|_| StegServiceError::FileError)?;

        decode_chunk(&mut buffer, generator_len)?;

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

//ret value of false means there were errors which RS was unable to fix
fn decode_chunk(chunk: &mut Vec<u8>, generator_len: u8) -> Result<bool, StegServiceError> {
    let generator_roots = get_roots_for_generator_with_len(generator_len);

    let mut syndromes: Vec<u8> = Vec::with_capacity(generator_len as usize);

    let mut found_error = false;
    for i in 0..generator_len {
        let mut sum = 0;
        let curr_root = generator_roots[i as usize];
        let mut adjusted_cur_root = 1;
        for j in 0..chunk.len() {
            sum ^= poly_mult(chunk[j], adjusted_cur_root);

            adjusted_cur_root = poly_mult(curr_root, adjusted_cur_root);
        }
        syndromes.push(sum);
        if sum != 0 {
            found_error = true;
        }
    }
    if !found_error {
        return Ok(true);
    }

    todo!()
}

fn encode_chunk(chunk: &mut Vec<u8>, generator: &[u8]) -> Result<(), StegServiceError> {
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
