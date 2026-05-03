use crate::services::galois_field::{EXP_TABLE, poly_mult_vecs};

//great name i know
//for len = 2 we expect alpha1(a1)=0 a2=1
//which represent (x+2^0)=0 and (x+2^1)=0
//we then mult then and expect to get (x^2+2x+x+2) or x^2+3x+2 (gf math replaces + with xor) which we represent in the arr as such
//arr[0] = 2 arr[1] = 3 arr[2] = 1
//this is the intended result of the generator for len = 2 note the ret len is len+1
pub fn generatae_generator(len: u8) -> Vec<u8> {
    let mut generator: Vec<u8> = vec![1, 1];

    for i in 1..len {
        //generate (x+2^i) in the opposite order tho cause x's power scales from low index to high
        let mut root = vec![1; 2];
        root[0] = EXP_TABLE[i as usize];

        generator = poly_mult_vecs(&generator, &root);
    }

    generator
}

pub fn get_roots_for_generator_with_len(len: u8) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(len as usize);
    for i in 0..len {
        result.push(EXP_TABLE[i as usize]);
    }
    result
}
