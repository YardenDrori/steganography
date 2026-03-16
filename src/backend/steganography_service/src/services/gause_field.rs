use std::{collections::HashMap, sync::LazyLock};

use rand::rand_core::UnwrapErr;

use crate::errors::steg_service_error::StegServiceError;

pub static EXP_TABLE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut exp_table: Vec<u8> = Vec::with_capacity(256);
    let mut value: u8 = 1;

    exp_table.push(1);
    for _ in 1..256 {
        value = polynomal_multiplication_by_two(value);
        exp_table.push(value);
    }
    return exp_table;
});

pub static LOG_TABLE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut log_table = vec![0u8; 256];

    for i in 0..256 {
        log_table[EXP_TABLE[i] as usize] = i as u8;
    }

    return log_table;
});

const POLYNOMAL_PRIME_NUMBER: u16 = 0x11d;
pub fn polynomal_multiplication_by_two(mut num: u8) -> u8 {
    //after shifting left we dont know if the number had an overflow after shifting so we check beforehand if it
    //will overflow
    if num & 0b10000000 == 0b10000000 {
        num <<= 1;
        num = (num as u16 ^ POLYNOMAL_PRIME_NUMBER) as u8;
        return num;
    }
    num <<= 1;
    return num;
}

pub fn poly_mult(num1: u8, num2: u8) -> u8 {
    if num1 == 0 || num2 == 0 {
        return 0;
    }
    let mut index_sum = (LOG_TABLE[num1 as usize] + LOG_TABLE[num2 as usize]) as u16;
    while index_sum > 255 {
        index_sum -= 255;
    }
    EXP_TABLE[index_sum as usize]
}

pub fn poly_div(numerator: u8, denominator: u8) -> u8 {
    if denominator == 0 {
        tracing::error!("Attempted to divide by zero in poly_div");
        panic!();
    }
    if numerator == 0 {
        return 0;
    }
    let mut index_sum =
        (LOG_TABLE[numerator as usize] as i16) - (LOG_TABLE[denominator as usize] as i16);
    while index_sum < 0 {
        index_sum += 255;
    }
    EXP_TABLE[index_sum as usize]
}

//i[0] == x^0, i[1] == x^1 i[2] == x^3...
//(3-4x+2x^2)*(2*4x+0x^2) -> (6+12x)+(-8x+16x^2)+(4x^2+8x^3) highest x power here is the len of
//m1-1 + len of m2-1
//result will be 8x^3+20x^2+4x+6 so four buckets
//res[0] = 6 res[1] = 4 res[2] = 20 res[3] = 8
pub fn poly_mult_vecs(num1: &[u8], num2: &[u8]) -> Vec<u8> {
    //we know its this length cause math is mathy
    let mut result: Vec<u8> = vec![0; num1.len() + num2.len() - 1];

    for i in 0..num1.len() {
        for j in 0..num2.len() {
            let mult_res = poly_mult(num1[i], num2[j]);
            //xor cause in GF plus/minus is replaced with xor. yeah i know its weird as fuck
            result[i + j] ^= mult_res;
        }
    }
    result
}

//NOTE This example is in REGULAR math! NOT GF(2^8)
//2x^3+x^2-6x-8  /  x-2
//2x^3 / x = 2x^2 -> div_res_vec = [0,0,2,0]
//div_res_vec*denominator: (denominator: [-2, 1, 0, 0])
//res=[0,0,-4,2]
//numerator[-8,-6,1,2] - res[0,0,-4,2] = [-8,-6,5,0] now we loop
pub fn poly_div_remainder_vecs(numerator: &[u8], denominator: &[u8]) -> Vec<u8> {
    let mut numer_clone = numerator.to_vec();

    for i in 0..numer_clone.len() {
        let pos = numer_clone.len() - i - 1;
        if pos < denominator.len() - 1 {
            //if we are here than its not divisible so we break
            break;
        }

        let div_res = poly_div(numer_clone[pos], denominator[denominator.len() - 1]);
        let mut div_res_vec = vec![0; numer_clone.len()];

        div_res_vec[pos - (denominator.len() - 1)] = div_res;

        let denom_mult_div_res = poly_mult_vecs(&div_res_vec, denominator);

        for i in 0..numer_clone.len() {
            numer_clone[i] ^= denom_mult_div_res[i];
        }
    }

    let res_vec = numer_clone[0..denominator.len() - 1].to_vec();

    res_vec
}
