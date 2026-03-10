use crate::errors::steg_service_error::StegServiceError;
use rand::{Rng, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

pub fn generate_unit_vector(seed: String, vec_size: usize) -> Result<Vec<f64>, StegServiceError> {
    //we need to hash it as rand only accepts u64 not strings
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let hashed_string = hasher.finalize();
    // we cant nor need to take more than 8 bytes so we just dump the rest
    let hashed_seed = u64::from_le_bytes(
        hashed_string[0..8]
            .try_into()
            .map_err(|_| StegServiceError::InvalidPayload)?,
    );

    let mut rng = StdRng::seed_from_u64(hashed_seed);

    let mut return_vec: Vec<f64> = Vec::with_capacity(vec_size);
    let mut squared_sum: f64 = 0.0;
    for _ in 0..vec_size {
        let value = rng.random_range(-1.0f64..1.0f64);
        squared_sum += value * value;
        return_vec.push(value);
    }

    //now we do fancy math to convert the vector to a unit vector (same direction but length in
    //space is 1) so that when we do dot with it and the coefficients the output will be normalized
    //(otherwise it could get too big or too small)
    //example:
    //delta = 10
    //bit=0 at: 0, 10, 20, 30...
    //bit=1 at: 5, 15, 25, 35...
    //now if we wanna hide bit 0 and the dot product is calcultaed
    //by u*c+u1*c1+u2*c2..
    //if u is twice as big for instance where we would normally get 9 we would get 18 and would
    //try to change the coefficients to match 20 as its the closest and while both 10 and 20 are bit=0
    //this isnt gaurenteed and even if it were we are doing more damage to the video quality for no
    //gain. thanks for coming to my TED-Talk
    let len_in_space = squared_sum.sqrt();

    // we are about to divide by this value so to avoid a panic we verify it first
    if len_in_space == 0.0 {
        return Err(StegServiceError::InvalidPayload);
    }

    for i in 0..vec_size {
        return_vec[i] /= len_in_space;
    }

    Ok(return_vec)
}

pub fn calculate_dot_product(coeffs: &[f64], unit_vector: &[f64]) -> Result<f64, StegServiceError> {
    if coeffs.len() != unit_vector.len() {
        return Err(StegServiceError::Other(
            "called \"calculate_dot_product\" with coeffs and unit arrays of differing lengths"
                .to_string(),
        ));
    }
    let mut sum = 0.0;
    for i in 0..coeffs.len() {
        sum += coeffs[i] * unit_vector[i];
    }
    Ok(sum)
}

pub fn do_back_projection_on_coeffs(
    coeffs: &mut [f64],
    unit_vector: &[f64],
    original_dot_operation_value: f64,
    modified_dot_operation_value: f64,
) -> Result<(), StegServiceError> {
    if coeffs.len() != unit_vector.len() {
        return Err(StegServiceError::Other(
            "called \"do_back_projection_on_coeffs\" with coeffs and unit arrays of differing lengths"
                .to_string(),
        ));
    }
    let dot_diff = modified_dot_operation_value - original_dot_operation_value;
    for i in 0..coeffs.len() {
        coeffs[i] += dot_diff * unit_vector[i];
    }
    Ok(())
}
