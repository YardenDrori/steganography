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
// mplace of x which will produce 2 syndromes for us if all syndromes's value are 0 we're done this
// is why we wanted division to with the generator in earlier steps to equate 0 cause the error
// formula is something like this e(x) = message*x + error*x so now if we plug in one of the values
// from the generator roots e.g 2^0 e(1) = message*1 + error*1 thanks to how we built the message
// we know that regardless of what the mssage was it is now 0 leaving us with e(1) = error*1 which
// is very usefull for us we can expand error to be Y*X^pos where Y is how off the byte is from
// what it was supposed to be and pos is in which recieved byte the error is the reason we do X to
// the power of pos instead of writing pos directly is because of how we represnt bytes on the
// encoding step. this doesnt tell us anything at all but these are the components of the error
// if we had more than 1 error the formula would be Y1*X^pos1 Y2*X^pos2 so we can make a polynom of
// all the errors called e(X) = Y1*X^pos1 + Y2*X^pos2 + ... Yn*X^posn
// where n is how many errors we had during transmission this is why every 2 parity bytes can fix 1
// error because every error is made of amount and position and each parity byte will allow us to
// recover 1 of those for reasons that will be explained later
// in the encoding step we represented each byte's index with a power of x so we stick to that
// pattern here if the error is in byte 4 than the err is Y(4)*X^4
// the genric formula is
// e(x) = Y0*X^pos0 + Y1*X^pos1 + ... Yn*X^posn
// now substituting X for the root to eliminate the message leaving us with just the error
// a generic formula for the syndrome is
// S[j] = Y*(2^j)^pos where pos is the error position and Y is its magnitude and j is the root power
// (this is the case because of how we encoded the message we encoded it in such a way that
// [1,2,3,4,5] represents 1+x+x^2+x^3+x^4 so we effectively encoded a function thus we can
// play around with X but we dont plug the root directly as we know its always 2 to the power of
// something so we just plug in that something) note we dont know any of these this is what we
// are tying to find we get S[j]
//
// -- MIGHT DELETE --
// as one value we can call Z
// but we do know that Z(j) = (2^j)^i*Y which is what we care about
// -- MIGHT DELETE --
//
//
// for this reason if S[1] = Y*(2^1)^pos
// and                S[2] = Y*(2^2)^pos = S[1]*2^pos
//                    S[3] = Y*(2^3)^pos = S[2]*2^pos
// we can see a pattern emerging S[i] = S[i-1]*2^pos (2 being the generator for GF(2^8))
// this will be usefull later however note this is only true if we have 1 error which we dont know
// as if we have more than 1 error we cant confirm what value of Y we are looking at Y1 Y2 or Y23..
//
// the reason this property exists is because of how we calculate the syndromes we take the message
// function plug in a root and each root gives us a different number that represents the error a
// good way to think of it is like having a function f(x) = x plugging x = 2 and x = 5 gives 2
// different numbers that represent the same thing now replace the f(x) with e(x) for the error
// function and we plug in the generator roots cause they clean up the formula by keeping only the
// error part of it as explained earlier so lets say we had 1 error in byte X^3 with a magnitude 7
// we plug in out two roots and get our two syndromes S1 and S2
// stated earlier each S represents Y*(2^j)^pos so we can deduce that S2/S1 = Y*(2^2)^pos / Y*(2^1)^pos
// which is equal to 2^pos now we have a number we know represents 2^pos so we just check it with
// the LOG_TABLE to see what power of 2 gives the number we got and what we find is the pos!
// now we just plug pos in the syndromes formula to find Y and we found the error
// note if we have more than 1 error this gets waaay more complicated cause now we have 4 equations
// with 4 paramaters so we cant simply divide them by each other like we just did while you can
// solve them and get the correct solution this is very inefficient and complex so we write an
// equation for sigma that makes it reasonable
//
// we define sigma as σ(X) = (1-2^pos1*X)(1-2^pos2*X) this means that plugging
// X = 2^-pos makes sigma = 0
// we did not derive this from anything this is just what we decided sigma is
//
// now we dont actually have σ(x) as if we did we would be done so to find it we use Berlekamp-Massey
//
// Berlekamp Massey works by establishing a rule that if we take sigma and expand it
// σ(x) = 1-2^pos2*X - 2^pos1*X + 2^pos1*X*2^pos2*X =
// σ(x) = 1 - 2^pos1*x
// σ(x) = 1 - 2^pos1*X - 2^pos2*X + 2^pos1*2^pos2*X^2 =
// now we can define 2^pos1 as A and 2^pos2 as B to simplify which gives us
// σ(x) = 1 - A*X - B*X + A*B*X^2 =
// σ(x) = 1 - X*(A+B) + A*B*X^2
//
//
// now if we take a look back at our syndromes
// S[j] = Y1*(2^j)^pos1 + Y2(2^j)^pos2 (for 2 errors)
// and we make A=2^pos1 and B=2^pos2 for simplicity like before we derive
// S[j] = Y1*A^j + Y2*B^j (lol BJ😏)
//
// if we plug 1,2,3 into S we see
// S[1] = Y1*A^1 + Y2*B^1
// S[2] = Y1*A^2 + Y2*B^2
// S[3] = Y1*A^3 + Y2*B^3
// S3 = S2*(A+B)-(A*B*S1) = (Y1*A^3 + Y2*B^2*A + Y1*A^2B + Y2*B^3) - A*B*S[1]
// Y1*A^3 + Y1*A^2*B + Y2*B^3 + Y2*B^2*A - (Y1A^2*B + Y2*B^2*A)
// leaving us with Y1*A^3+Y2*B^3
// we can again see a pattern emerge we S[i] = S[i-1]*(A+B) - S[i-2]*A*B
// yeah i hate this formula so much
//
// now idk what deal with the devil was made to get this fomula to work but here is an explanation
// we can clearly see in the three examples that what changes is the power of A and B but we dont
// have an easier way to achieve that mathematically other than this ritual with the devil
//
// we further abstract even A and B to σ1 and σ2
// we abstract them as σ1 = (A+B) and σ2 = (A*B)
// and if u recall our earlier formula
// σ(x) = 1 - X*(A+B) + A*B*X^2
// replacing A and B with our sigma values gives us
// σ(x) = 1 - σ1*X+σ2*X^2
// so σ1 and σ2 represent numbers we dont know from multiplying they are just unkown
// this gives us the updated recurrence of
// S[i] = S[i-1] * σ1 - S[i-2] * σ2
// NOTE: this is an example for 2 errors the formula varries on how many errors there are
// for each error we have more syndromes and more sigmas
//
// now you do Berlekamp-Massey which is a fancy algorithm in which you guess the recurrence
// the first guess is S[j] = 0 aka there are no errors
// if this guess is false well update our guess as will be seen in a bit
// now as we derived just abbove we have this formula (but for 1 error earlier we assumed 2 it can
// expand and shrink dynamically and infintely as needed)
// S[j] = σ1 * S[j-1]
// this formula assumes there is 1 error and only 1 error
// now we plug in 1 and 2 in the message received on our end to find the first and second
// syndromes as this formula assumes 1 error and we need 2 syndromes to fix 1 error
// we take those two syndromes which *should* make the formula shown abbove
// we use this formula and the 2 syndromes to find the value of sigma
// now after we find its value we check if sigma can predict the next syndrome with the same
// formula we just used to find it if it can we're done with this step if it isnt we increase the err count
// our formula assumes so now it would be
// S(i) = S[i-1]*σ1 - S[i-2] * σ2 (notice the pattern for every error we just dec but an ever
// decreasing S[i] value and multiply by a new unkown (sigma)
//
// after we do that we dont throw away our old work of finding sigma as we can still use it
// to help us make another guess for the new σ1 and σ2 more efficiently than starting from scratch
// we do this by taking the discrepency between what we expected to get S[i] (in this case S[3])
// and what we got from doing the formula with the prev σ we call this discrepency delta 𝛿
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
