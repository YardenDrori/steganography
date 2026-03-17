use crate::errors::steg_service_error::StegServiceError;

pub fn berlecamp_massey(syndromes: &Vec<u8>) -> Result<(), StegServiceError> {
    let mut curr_guess: u8 = 0;

    loop {
        for i in 1..syndromes.len() {
            if syndromes[i - 1] * curr_guess != syndromes[i] {
                break;
            }
        }

        break;
    }

    todo!()
}
