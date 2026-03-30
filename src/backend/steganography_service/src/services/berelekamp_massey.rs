use crate::{
    errors::steg_service_error::StegServiceError,
    services::galois_field::{poly_div, poly_mult, poly_mult_vecs},
};

pub fn berlecamp_massey(syndromes: &Vec<u8>) -> Result<(), StegServiceError> {
    let mut lambda = vec![1u8]; //Λ
    let mut prev_lambda = vec![1u8]; //B
    let mut prev_delta: u8 = 1; //b

    // obviously this is shift lol we init it as 0 to inc it at the start
    // of each loop iteration to save us from updating it at the end of
    // the loop which would make us need to remember updating it at
    // multiple places
    let mut shift = 0u8;

    let mut err_count: u8 = 0; //L

    let mut i = 0;
    loop {
        shift += 1;
        let mut delta = 0;

        for j in 0..i + 1 {
            // S[0] - Λ[1]*S[-1] = 0?
            // we need to handle cases of out of bounds access
            let coeff = match lambda.get(j + 1) {
                Some(v) => v.clone(),
                None => 0,
            };
            let syndrome = match syndromes.get(j + 1) {
                Some(v) => v.clone(),
                None => 0,
            };

            delta ^= poly_mult(coeff, syndrome);
        }
        if delta == 0 {
            continue;
        }

        // X^shift * B = adjusted_B
        let mut adjusted_prev_lambda = vec![0u8, shift];
        adjusted_prev_lambda.append(&mut prev_lambda);

        prev_lambda = lambda.clone();

        // δ/b * adjusted_B = adjusted_scaled_B
        for coeff in adjusted_prev_lambda.iter_mut() {
            let multiplier = poly_div(delta, prev_delta);
            *coeff = poly_mult(*coeff, multiplier);
        }

        //  Λ_old - adjusted_scaled_B = Λ_new
        let len = if adjusted_prev_lambda.len() > lambda.len() {
            adjusted_prev_lambda.len()
        } else {
            lambda.len()
        };
        for j in 0..len {
            let curr_lambda_val = match lambda.get(j) {
                Some(v) => v.clone(),
                None => 0,
            };
            let prev_lambda_val = match adjusted_prev_lambda.get(j) {
                Some(v) => v.clone(),
                None => 0,
            };
        }

        i += 1;
        break;
    }

    todo!()
}
