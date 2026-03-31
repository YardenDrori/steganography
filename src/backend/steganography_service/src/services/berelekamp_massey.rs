use crate::{
    errors::steg_service_error::StegServiceError,
    services::galois_field::{poly_div, poly_mult},
};

pub fn berlecamp_massey(syndromes: &Vec<u8>) -> Result<Vec<u8>, StegServiceError> {
    let mut lambda = vec![1u8]; //Λ
    let mut prev_lambda = vec![1u8]; //B
    let mut prev_delta: u8 = 1; //b

    // obviously this is shift lol we init it as 0 to inc it at the start
    // of each loop iteration to save us from updating it at the end of
    // the loop which would make us need to remember updating it at
    // multiple places
    let mut shift = 0u8;

    let mut err_count: u8 = 0; //L

    for i in 0..syndromes.len() {
        shift += 1;
        let mut delta = 0;

        for j in 0..i + 1 {
            // S[0] - Λ[1]*S[-1] == 0?
            // we need to handle cases of out of bounds access
            let coeff = match lambda.get(j) {
                Some(v) => v.clone(),
                None => 0,
            };
            let syndrome = match syndromes.get(i - j) {
                Some(v) => v.clone(),
                None => 0,
            };

            delta ^= poly_mult(coeff, syndrome);
        }
        if delta == 0 {
            continue;
        }

        // X^shift * B = adjusted_B
        let mut adjusted_prev_lambda = vec![0u8; shift as usize];
        adjusted_prev_lambda.extend_from_slice(&prev_lambda);

        //we do this check here to save allocating memory to a temp vec which isnt that
        //cheap. and we do this workaround with the delta as we still need the old value but
        //cloning a number is cheap so we're fine with it
        let prev_delta_to_use = prev_delta;
        if err_count * 2 < i as u8 + 1 {
            prev_lambda = lambda.clone();
            prev_delta = delta;
            err_count = i as u8 + 1 - err_count;
            shift = 0;

            if err_count > (syndromes.len() / 2) as u8 {
                return Err(StegServiceError::ReedSolomonError(
                    "Too many errors to fix.".to_string(),
                ));
            }
        }

        // δ/b * adjusted_B = adjusted_scaled_B
        let multiplier = poly_div(delta, prev_delta_to_use);
        for coeff in adjusted_prev_lambda.iter_mut() {
            *coeff = poly_mult(*coeff, multiplier);
        }

        //  Λ_old - adjusted_scaled_B = Λ_new
        if lambda.len() < adjusted_prev_lambda.len() {
            lambda.resize(adjusted_prev_lambda.len(), 0u8);
        }
        for j in 0..adjusted_prev_lambda.len() {
            lambda[j] ^= adjusted_prev_lambda[j];
        }
    }

    Ok(lambda)
}
