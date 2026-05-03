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

// to encode we use the generator element (2) to make a generator polynomal so if we wanted 5
// parity bytes the generator would be g(x) = (x-2^0)*(x-2^1)*(x-2^2)*(x-2^3)*(x-2^4) multiplying
// them will give us a formula that plugging any value from 2^0 to 2^4 makes g(x) = 0 this is
// usefull for for reasons that will be explained later, after we have the generator we take the
// original message pad a zero for each parity byte we said we wanted and divide that with the
// generator using long division we take whatever remainder is left which is gauaranteed to be at
// most as big as parity bytes because math is cool like that, we replace the zeros with the
// remainders this gives us the benifit of division with the generator now giving us zero which
// will be very usefull for decoding we are now done with the encoding step

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

// for this reason if S[1] = Y*(2^1)^pos
// and                S[2] = Y*(2^2)^pos = S[1]*2^pos
//                    S[3] = Y*(2^3)^pos = S[2]*2^pos
// we can see a pattern emerging S[i] = S[i-1]*2^pos (2 being the generator for GF(2^8))
// this will be usefull later however note this is only true if we have 1 error which we dont know
// as if we have more than 1 error we cant confirm what value of Y we are looking at Y1 Y2 or Y23..

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

// we define lambda as Λ(X) = (1-2^pos1*X)(1-2^pos2*X) this means that plugging
// X = 2^-pos makes lambda = 0
// we did not derive this from anything this is just what we decided lambda is

// now we dont actually have Λ(x) as if we did we would be done so to find it we use Berlekamp-Massey

// Berlekamp Massey works by establishing a rule that if we take sigma and expand it
// Λ(x) = 1-2^pos2*X - 2^pos1*X + 2^pos1*X*2^pos2*X =
// Λ(x) = 1 - 2^pos1*x
// Λ(x) = 1 - 2^pos1*X - 2^pos2*X + 2^pos1*2^pos2*X^2 =
// now we can define 2^pos1 as A and 2^pos2 as B to simplify which gives us
// Λ(x) = 1 - A*X - B*X + A*B*X^2 =
// Λ(x) = 1 - X*(A+B) + A*B*X^2

// now if we take a look back at our syndromes
// S[j] = Y1*(2^j)^pos1 + Y2(2^j)^pos2 (for 2 errors)
// and we make A=2^pos1 and B=2^pos2 for simplicity like before we derive
// S[j] = Y1*A^j + Y2*B^j (lol BJ😏)

// if we plug 1,2,3 into S we see
// S[1] = Y1*A^1 + Y2*B^1
// S[2] = Y1*A^2 + Y2*B^2
// S[3] = Y1*A^3 + Y2*B^3
// S3 = S2*(A+B)-(A*B*S1) = (Y1*A^3 + Y2*B^2*A + Y1*A^2B + Y2*B^3) - A*B*S[1]
// Y1*A^3 + Y1*A^2*B + Y2*B^3 + Y2*B^2*A - (Y1A^2*B + Y2*B^2*A)
// leaving us with Y1*A^3+Y2*B^3
// we can again see a pattern emerge we S[i] = S[i-1]*(A+B) - S[i-2]*A*B
// yeah i hate this formula so much

// now idk what deal with the devil was made to get this fomula to work but here is an attempt at
// an explanation:
// we can clearly see in the three examples that what changes is the power of A and B but we dont
// have an easier way to achieve that mathematically other than this ritual with the devil

// we further abstract even A and B to σ1 and σ2
// we abstract them as σ1 = (A+B) and σ2 = (A*B)
// and if u recall our earlier formula
// Λ(x) = 1 - X*(A+B) + A*B*X^2
// replacing A and B with our sigma values gives us
// Λ(x) = 1 - σ1*X + σ2*X^2
// so σ1 and σ2 represent numbers we dont know from multiplying they are just unkown
// this gives us the updated recurrence of
// S[i] = S[i-1] * σ1 - S[i-2] * σ2
// this is called a linear recurrence a famous one for instance is the fibonacci sequence
// S[i] = S[i-1]*1 + S[i-2]*1 so for the fibonacci sequence
// Λ(x) = [-1,1] (technically [1,-1,1] but well get into it later)
// NOTE: this is an example for 2 errors the formula varries on how many errors there are
// for each error we have more syndromes and more sigmas
// NOTE: the signs are alternating starting from positive so for two erros as seen here its + - +
// (1 - σ1*X + σ2*X^2)
// if we were to add a third error it would be
// +1 - σ1*X + σ2*X^2 - σ3*X^3
// and so on...

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

// after we do that we dont throw away our old work of finding sigma as we can still use it
// to help us make another guess for the new σ1 and σ2 more efficiently than starting from scratch
// doing this involves this formula
// Λ_new = Λ_old - δ/b * B * x^shift
// Λ_old is self explanetory the curr Λ value we wanna update
// delta is the difference from the correct syndrome and the guessed syndrome so actual - guessed =
// delta
// B is the last working Λ before we increased the estimated error count
// b is the delta that B gave at its failure point
// shift is how many steps ago was B(the now old lambda) our current lambda or how many steps ago
// did we abandon it
// additionally on the first run B and lambda are initialized as [1] and b is initialized as 1
// we increase the estimated error count when the curr_guess is over 2*how many errors we think
// there are
// Breaking down WHY this formula works is a bit of a pain but its as follows

// NOTE: from now on we'll treat the index 0 of
// lambda as not a sigma value but as a field thats just there so the math works so sigma1 = 2 here
// sigma2 = 3 here etc same for B as otherwise they would be null and the formula wont work on
// first run with null nor can we interpert them as 0 as it would also not work so we set them 1

// and our syndromes are [0, 1, 1, 2, 3, 5, 8] (this is the fibonacci sequence)
// so we define the following variables
// B - previous lambda when we increased L (unintorudced yet)
// b - the delta of the B at the time it was our main lambda at its last failure point
// L - how many errors we think there are
// shift - how many loop iterations (loop unintroduced yet) has it been since B was updated
// with the following defaults when starting
// Λ = [1] -- again we ignore index 0 its for math stuff
// δ = 1
// B = [1] --same as lambda here
// b = 1
// L = 0 --we assume no errors
// i - curr iteration of the loop = 1
// shift = 1 -- we just started and just "updated" (created) B and when we update B we set shift to 1

// so we compare S[0] - Λ[1] = 0? -> 0 - 0 = 0? -> TRUE!
// lambda delta B and b remain unchanged
// i = i + 1 = 2 first iteration done
// L - remaisn unchanged as we werent wrong so far the guess holds true
// shift increases as its been another iteration with no update to B so its now 2
// we now check if the rule continues being true
// S[1] - Λ[1] = 0? -> 1-0 = 0? -> FALSE!
// as we were false we NOW check the condition if n > L*2 and as L=0 its true
// thus L=L+1 = 1 -- we now assume 1 error exists
// we save the deviation from expected value (1-0 = 1) in delta
// now as we were wrong we update the lambda polynomal
// Λ_new = Λ_old - δ/b * B * X^shift --reasoning for why this formula works will be at the end.
// Λ_new = [1] - 1/1 * [1] * X^2 = [1] - [0,0,-1] = [1,0,1]
// additionally as L increased we updated B and b
// B = [1] b = 1 and shift goes back down to 1 as B was just updated
// L = 1 -> 1*2 < n(2)? -> FALSE L stays at 1
// iteration 3 now (i=3)

// S[2] + S[1]*0 - 1*S[0] = 0? -> 1 + 0 - 1 = 0 == 0 -> TRUE!
// as this is true we keep B b lambda and delta untouched
// increase shift by 1
// dont check the L condition as we are right
// and move on
// S[3] + 0*S[2] - 1*S[1] = 0? -> 2 + 0 - 1 = 1 != 0 -> FALSE!
// as this is false we now again first update the lambda
// Λ_new = [1,0,1] - 1/1 * [1] * X^2 = [1,0,1] - [0,0,1] yeah this is a loop im lost here

// After the Berlekamp-Massey charade we have a polynomal that plugging 2^any_error_pos = 0
// and with that polynomal we represented 2^pos = 0 i.e if we plug 2 to the power of a position
// with an error the lambda value will be 0 (this is just how we defined lambda earlier and now
// that we have it we can move on) so to find the pos values its not that flashy we just brute
// force all 256 possible powers of 2 and save the ones that output 0 we use the EXP_TABLE oc to
// save compute tho

// then we move on to Frein's algorithm which helps us extract the magnitude of the error now that
// we know where it occured
// we recall these parts from way abbove
// S[j] = Y1*(2^j)^pos1 + Y2(2^j)^pos2 (for 2 errors)
// which contains info about the magnitudes! but its all tangled up each syndrome is composed of
// all the errors from all the bytes with errors so we need to isolate the magnitudes
// we have a buncha syndrome leftover to make formulas to extract them but doing a
// buncha 7th grade math problems but that is expensive, annoying, and slow so we use a shortcut
// lets do first an example for a single error
// our syndromes are S1 = Y1*2^pos1.
// we can know S1 and pos1 so to find Y1 we just divide S1 with 2^pos isolating Y1 giving us the
// magnitude
// now, this doesnt scale (unlike ur mom)
// for 2 errors we have the syndromes S1 = Y1*2^pos1 + Y2*2^pos2, S2 = Y1*2^2*pos1 + Y2*2^2*pos2
// now we can again either do a 7th grade algebra problem which is O(n^3) orrrrrrr we can
// do S2-S1*(2^pos2)
// as we did before lets abstract 2^pos1 to A and 2^pos2 to B
// S2 - S1*B = Y1*A^2 + Y2*B^2 - Y1*A*B - Y2*B^2
// Y1*A^2 - Y1*A*B = S2-S1
// notice Y2 has been completely eliminated from the formula so this is back to a signel variable
// equation which we can easily solve

// lets do 3 errors next
// S1 = Y1*A + Y2*B + Y3*C
// S2 = Y1*A^2 + Y2*B^2 + Y3*C^2
// S3 = Y1*A^3 + Y2*B^3 + Y3*C^3
// we need to eliminate both Y2 and Y3 here same as we did earlier
// we do this by finding the ratio between the syndromes notice in the 2 variable example we chose
// to multiply with B (2^pos2) and that is
// due to Y2 being multiplied by B so its scaling factor is B
// (
// Y2*B^2 / Y2*B = B
// )
// so for 3 variables we see that Y2 scales with B and Y3 scales with C
// S3 - S2*C
// Y1*A^3 + Y2*B^3 + Y3*C^3 - Y1*A^2*C - Y2*B^2*C - Y3*C^3 = S3-S2*C
// S3 - S2*C = Y1*A^3 + Y2*B^3 - Y1*A^2*C - Y2*B^2*C
// notice we deleted Y3 but our goal is to remove both Y3 and Y2 but we can only remove one
// variable at a time and need two formulas per removal so we cant remove Y2 yet (without also
// having Y3 still involved) thus we need another formula without Y3 which we get this way
// S2 - S1*C = Y1*A^2 + Y2*B^2 + Y3*C^2 - Y1*AC - Y2*BC - Y3*C^2
// S2 - S1*C = Y1*A^2 + Y2*B^2 - Y1*AC - Y2*BC
// once again Y3 is removed and we now have two formulas without Y3 so we can move on to remove Y2
// S3-S2*C-(S2-S1*C)*B=Y1*A^3+Y2*B^3-Y1*A^2*C-Y2*B^2*C-(Y1*A^2*B+Y2*B^3-Y1*ABC-Y2*B^2C)
// Y1*A^3 + Y2*B^3 - Y1*A^2*C - Y2*B^2*C - Y1*A^2 - Y2*B^3 + Y1*ABC + Y2*B^2C
// Y1*A^3 - Y1*A^2*C - Y1*A^2 + Y1*ABC
// Y1(A^3 - A^2*C + ABC)
// now we only have Y1s in here so we can again solve the formula

// NOTE: HOWEVER this is still O(n^3) as we for each error (n) we need to do a O(n^2) to find the
// position this is essentially yhe normal way to solve this
//
// what Forney's algo does is generate us two polynomals that produce the numerator and denominator
// for the final division for instance with 2 errors we got
// S2 - B·S1 = Y1(A²-AB)
// Y1 = (S2 - B·S1) / (A²-AB)
// so one polynomal will find us (S2 - B·S1) and another will get for us (A²-AB)

// we first define a polynomal which just has all our syndromes as coefficients
// S(x) = S0*X^0 + S1*X^1 + S2*X^3 + S3*X^4 ... Sn*X^n
// we then define a polynomal OMEGA where omega is Λ(x) * S(x) mod 2x^t where t is err_count
// to explain why we define Omega as such lets see what omega is for 2 errors
// for 2 errors we will have the lambda lambda
// Λ(x) = (1 - 2^pos1·x) -- this is the same as 1 - σ1*x
// calling 2^pos1 A like we did before
// Λ(x) = (1 - A·x)
// and a syndrome polynomal of
// S(x) = S1 + S2*x
// we can expand each syndrome
// S[1] = Y*A, S[2] = Y*A
// thus S(x) = Y*A + Y*A^2*X
// multiplying the syndromes polynomal with the lambda polynomal yields us
// S(x)*Λ(x) = (Y*A + Y*X*A^2) * (1 - A*X)
// S(x)*Λ(x) = Y*A - Y*X*A^2 + Y*X*A^2 - Y*X^2*A^3
// S(x)*Λ(x) = Y*A - Y*X^2*A^3
// we now do the mod which results in us dropping Y*X^2*A^3 TODO: explain why
// S(x)*Λ(x) = Y*A
// notice that this is equal to S[1]
// this scales to any number of errors we know this to be true due to how we our syndromes
// NOTE: this is where I stopped i dunnow why this holds true despite doing the math for 1 error
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
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
        berelekamp_massey::berlecamp_massey,
        galois_field::{EXP_TABLE, poly_div, poly_div_remainder_vecs, poly_mult, poly_mult_vecs},
        rs_generator_vec::{generatae_generator, get_roots_for_generator_with_len},
    },
};

pub fn reed_solomon_encode(
    payload_path: PathBuf,
    configs: EmbedConfigs,
) -> Result<(), StegServiceError> {
    if configs.reed_solomon_padding_byte_count > 254 {
        return Err(StegServiceError::InvalidPayload);
    }
    if configs.reed_solomon_padding_byte_count == 0 {
        return Ok(()); // passthrough — no parity, file unchanged
    }

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

        // After encode_chunk, buffer grows from data_len to data_len + parity bytes
        // (parity bytes are placed at the front: [parity..., data...]).
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
    if configs.reed_solomon_padding_byte_count > 254 {
        return Err(StegServiceError::InvalidPayload);
    }
    if configs.reed_solomon_padding_byte_count == 0 {
        return Ok(()); // passthrough — file is already plain bytes
    }

    let output_path = format!("{}_decoded", &payload_path.display());

    let input_file_pointer = File::open(&payload_path).map_err(|_| StegServiceError::FileError)?;
    let output_file_pointer =
        File::create(&output_path).map_err(|_| StegServiceError::FileError)?;

    let mut payload_bytes_left = input_file_pointer
        .metadata()
        .map_err(|_| StegServiceError::FileError)?
        .len() as i128;

    let mut reader = BufReader::new(input_file_pointer);
    let mut writer = BufWriter::new(output_file_pointer);

    let generator_len = configs.reed_solomon_padding_byte_count;
    let parity = generator_len as usize;

    while payload_bytes_left > 0 {
        // Encoded chunks on disk are up to 255 bytes (data + parity), not (255 - parity).
        let read_len = std::cmp::min(255i128, payload_bytes_left) as usize;
        if read_len <= parity {
            return Err(StegServiceError::ReedSolomonError(
                "encoded chunk smaller than parity count".to_string(),
            ));
        }
        let mut buffer = vec![0u8; read_len];

        reader
            .read_exact(&mut buffer)
            .map_err(|_| StegServiceError::FileError)?;

        let recovered = decode_chunk(&mut buffer, generator_len)?;
        if !recovered {
            tracing::warn!(
                "RS chunk had uncorrectable errors; passing through with possible corruption"
            );
        }

        // Strip the parity prefix; only the data tail is the recovered payload.
        writer
            .write_all(&buffer[parity..])
            .map_err(|_| StegServiceError::FileError)?;

        payload_bytes_left -= read_len as i128;
    }

    writer.flush().map_err(|_| StegServiceError::FileError)?;

    std::fs::remove_file(&payload_path).map_err(|_| StegServiceError::FileError)?;
    std::fs::rename(output_path, payload_path).map_err(|_| StegServiceError::FileError)?;
    Ok(())
}

/// In-memory RS encode for small fixed-size buffers (e.g. the bit-stream header).
/// Caller's `buf` must satisfy `buf.len() + parity <= 255`.
pub fn rs_encode_in_place(buf: &mut Vec<u8>, parity: u8) -> Result<(), StegServiceError> {
    if parity > 254 {
        return Err(StegServiceError::InvalidPayload);
    }
    if parity == 0 {
        return Ok(());
    }
    let g = generatae_generator(parity);
    encode_chunk(buf, &g)
}

/// In-memory RS decode for small fixed-size buffers (e.g. the bit-stream header).
/// On uncorrectable corruption, returns Ok(false) and leaves `buf` as-is so the caller
/// can decide whether to fail or pass through the parity-stripped payload.
pub fn rs_decode_in_place(buf: &mut Vec<u8>, parity: u8) -> Result<bool, StegServiceError> {
    if parity > 254 {
        return Err(StegServiceError::InvalidPayload);
    }
    if parity == 0 {
        return Ok(true);
    }
    decode_chunk(buf, parity)
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

    // Berlekamp-Massey: "too many errors" is best-effort uncorrectable, not a hard error.
    let lambda = match berlecamp_massey(&syndromes) {
        Ok(l) => l,
        Err(StegServiceError::ReedSolomonError(_)) => {
            tracing::warn!("RS chunk uncorrectable (BM reports too many errors)");
            return Ok(false);
        }
        Err(e) => return Err(e),
    };

    //lambda always has err_count+1 elements
    if lambda.len() == 1 {
        tracing::warn!("lambda contains no meaningful values");
        return Ok(true);
    }

    // Chien search: roots of Λ(x). Range 0..255 covers each non-zero α^i exactly once
    // (α^0 == α^255 in GF(2^8)).
    // Λ(x) = 1 + σ1*X + σ2*X^2 + ...
    let mut error_poitions: Vec<u8> = vec![];
    for i in 0..255usize {
        let root = EXP_TABLE[i];
        let mut curr_root = root;

        //sum = Λ(root) = 1 + σ1*root + σ2*root^2 + σ3*root^3...
        let mut sum = 1;
        for j in 1..lambda.len() {
            sum ^= poly_mult(curr_root, lambda[j]);
            curr_root = poly_mult(curr_root, root);
        }
        if sum == 0 {
            error_poitions.push(i as u8);
        }
    }

    // Chien must surface deg(Λ) roots, otherwise the syndromes are inconsistent.
    if error_poitions.len() != lambda.len() - 1 {
        tracing::warn!(
            "RS chunk uncorrectable (Chien found {} roots, expected {})",
            error_poitions.len(),
            lambda.len() - 1
        );
        return Ok(false);
    }

    // ===== Forney's algorithm — compute error magnitudes =====
    // Ω(x) = (S(x) · Λ(x)) mod x^(2t), 2t = generator_len
    let mut omega = poly_mult_vecs(&syndromes, &lambda);
    omega.truncate(generator_len as usize);

    // Λ'(x) — formal derivative. In char 2, even-indexed terms drop, so
    // Λ'[j] = lambda[j+1] when (j+1) is odd (i.e. when j is even), else 0.
    let mut lambda_prime: Vec<u8> = vec![0u8; lambda.len() - 1];
    for j in 0..lambda_prime.len() {
        if (j + 1) % 2 == 1 {
            lambda_prime[j] = lambda[j + 1];
        }
    }

    for &i in &error_poitions {
        // Λ has roots at α^(-pos), so pos = (255 - i) mod 255
        let pos = ((255u32 - i as u32) % 255) as usize;
        if pos >= chunk.len() {
            tracing::warn!(
                "RS chunk uncorrectable (error pos {} >= chunk.len {})",
                pos,
                chunk.len()
            );
            return Ok(false);
        }
        let alpha_i = EXP_TABLE[i as usize]; // = X_k_inv = α^(-pos)

        // evaluate Ω(α^i)
        let mut x_pow = 1u8;
        let mut omega_at = 0u8;
        for k in 0..omega.len() {
            omega_at ^= poly_mult(omega[k], x_pow);
            x_pow = poly_mult(x_pow, alpha_i);
        }
        // evaluate Λ'(α^i)
        let mut x_pow = 1u8;
        let mut lp_at = 0u8;
        for k in 0..lambda_prime.len() {
            lp_at ^= poly_mult(lambda_prime[k], x_pow);
            x_pow = poly_mult(x_pow, alpha_i);
        }
        if lp_at == 0 {
            tracing::warn!("RS chunk uncorrectable (Λ'(X⁻¹) = 0)");
            return Ok(false);
        }
        // Generator roots start at α^0 (j₀ = 0), char 2:
        //   Y_k = X_k · Ω(X_k_inv) / Λ'(X_k_inv), with X_k = α^pos
        let x_pos = EXP_TABLE[pos];
        let y = poly_div(poly_mult(x_pos, omega_at), lp_at);
        chunk[pos] ^= y;
    }

    Ok(true)
}

fn encode_chunk(chunk: &mut Vec<u8>, generator: &[u8]) -> Result<(), StegServiceError> {
    let data_len = chunk.len();
    let parity = generator.len() - 1;

    //not 256 cause we cant use 0 as LOG_TABLE[0] is undefined
    if parity + data_len > 255 {
        return Err(StegServiceError::ReedSolomonError(
            "remainder and chunk size sum exceeds 255".to_string(),
        ));
    }

    // Systematic RS: c(x) = m(x)*x^parity + r(x), r(x) = m(x)*x^parity mod g(x).
    // In low-power-first array form we shift data up by xᵖ by prepending parity zeros.
    let mut shifted = vec![0u8; parity];
    shifted.extend_from_slice(chunk);

    let remainder = poly_div_remainder_vecs(&shifted, generator);

    // Codeword layout: chunk = [r_0..r_{p-1}, d_0..d_{k-1}]
    chunk.clear();
    chunk.extend_from_slice(&remainder);
    chunk.extend_from_slice(&shifted[parity..]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encoded chunk must satisfy c(α^j) = 0 for j = 0..parity (definition of a codeword).
    fn assert_is_codeword(chunk: &[u8], parity: u8) {
        let roots = get_roots_for_generator_with_len(parity);
        for j in 0..parity as usize {
            let mut sum = 0u8;
            let mut x_pow = 1u8;
            for &c in chunk {
                sum ^= poly_mult(c, x_pow);
                x_pow = poly_mult(x_pow, roots[j]);
            }
            assert_eq!(sum, 0, "c(α^{}) ≠ 0 — not a valid codeword", j);
        }
    }

    #[test]
    fn encode_produces_valid_codeword_single_byte() {
        let g = generatae_generator(2);
        let mut chunk = vec![5u8];
        encode_chunk(&mut chunk, &g).unwrap();
        assert_eq!(chunk.len(), 3, "data 1 + parity 2");
        assert_is_codeword(&chunk, 2);
        // Layout check: data tail = 5, parity prefix = 2 bytes
        assert_eq!(chunk[2], 5);
    }

    #[test]
    fn encode_produces_valid_codeword_long() {
        let parity = 16u8;
        let g = generatae_generator(parity);
        let mut chunk: Vec<u8> = (0..200).map(|i| (i * 7 + 13) as u8).collect();
        let original = chunk.clone();
        encode_chunk(&mut chunk, &g).unwrap();
        assert_eq!(chunk.len(), 200 + parity as usize);
        assert_is_codeword(&chunk, parity);
        // Data must be in the tail
        assert_eq!(&chunk[parity as usize..], original.as_slice());
    }

    #[test]
    fn roundtrip_no_errors() {
        let parity = 16u8;
        let g = generatae_generator(parity);
        let original: Vec<u8> = (0..100).map(|i| (i * 31 + 5) as u8).collect();
        let mut chunk = original.clone();
        encode_chunk(&mut chunk, &g).unwrap();

        let recovered = decode_chunk(&mut chunk, parity).unwrap();
        assert!(recovered);
        assert_eq!(&chunk[parity as usize..], original.as_slice());
    }

    #[test]
    fn roundtrip_corrects_8_errors_with_parity_16() {
        let parity = 16u8;
        let g = generatae_generator(parity);
        let original: Vec<u8> = (0..200).map(|i| (i * 17 + 3) as u8).collect();
        let mut chunk = original.clone();
        encode_chunk(&mut chunk, &g).unwrap();
        let codeword_len = chunk.len();

        // Inject 8 byte errors at distinct positions (max correctable for parity=16)
        let positions = [0usize, 5, 17, 50, 100, 137, 180, codeword_len - 1];
        for &p in &positions {
            chunk[p] ^= 0x42;
        }

        let recovered = decode_chunk(&mut chunk, parity).unwrap();
        assert!(recovered, "8 errors should be correctable with parity 16");
        assert_eq!(&chunk[parity as usize..], original.as_slice());
    }

    #[test]
    fn nine_errors_with_parity_16_returns_uncorrectable() {
        let parity = 16u8;
        let g = generatae_generator(parity);
        let mut chunk: Vec<u8> = (0..200).map(|i| (i * 11 + 1) as u8).collect();
        encode_chunk(&mut chunk, &g).unwrap();
        let codeword_len = chunk.len();

        // 9 errors > floor(parity / 2) = 8 — uncorrectable
        let positions = [0usize, 5, 17, 50, 100, 137, 180, 200, codeword_len - 1];
        for &p in &positions {
            chunk[p] ^= 0x99;
        }

        let recovered = decode_chunk(&mut chunk, parity).unwrap();
        assert!(!recovered, "9 errors should not be correctable with parity 16");
    }

    #[test]
    fn helpers_passthrough_for_parity_zero() {
        let mut buf = vec![1, 2, 3, 4, 5];
        let original = buf.clone();
        rs_encode_in_place(&mut buf, 0).unwrap();
        assert_eq!(buf, original, "parity=0 encode is a no-op");
        assert!(rs_decode_in_place(&mut buf, 0).unwrap());
        assert_eq!(buf, original, "parity=0 decode is a no-op");
    }

    #[test]
    fn header_roundtrip_8_bytes_plus_parity() {
        // Mirrors the bit-stream header path: encode 8 plain bytes with parity, decode back.
        let plain: u64 = 1234567890123;
        let parity = 16u8;

        let mut header = plain.to_le_bytes().to_vec();
        rs_encode_in_place(&mut header, parity).unwrap();
        assert_eq!(header.len(), 8 + parity as usize);

        // Inject 4 byte errors (well within parity=16 budget)
        header[0] ^= 0xAA;
        header[7] ^= 0xBB;
        header[15] ^= 0xCC;
        header[20] ^= 0xDD;

        let recovered = rs_decode_in_place(&mut header, parity).unwrap();
        assert!(recovered);

        let parity_usize = parity as usize;
        let recovered_plain = u64::from_le_bytes(
            header[parity_usize..parity_usize + 8].try_into().unwrap(),
        );
        assert_eq!(recovered_plain, plain);
    }

    #[test]
    fn errors_at_every_position_with_max_parity_density() {
        // parity 6, data 10 → can correct 3 errors. Verify positions all the way through chunk.
        let parity = 6u8;
        let g = generatae_generator(parity);
        for err_pos in [0usize, 1, 2, 5, 8, 10, 13, 15] {
            let original: Vec<u8> = (0..10).map(|i| (i * 19 + 7) as u8).collect();
            let mut chunk = original.clone();
            encode_chunk(&mut chunk, &g).unwrap();
            chunk[err_pos] ^= 0x77;
            let recovered = decode_chunk(&mut chunk, parity).unwrap();
            assert!(
                recovered,
                "1 error at pos {} should be correctable with parity 6",
                err_pos
            );
            assert_eq!(
                &chunk[parity as usize..],
                original.as_slice(),
                "data mismatch after correcting error at pos {}",
                err_pos
            );
        }
    }
}
