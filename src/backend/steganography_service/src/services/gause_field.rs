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

// fn poly_div_vecs(numerator: &[u8], denominator: &[u8]) -> Vec<u8> {
//     let mut remainder_vec: Vec<u8> = Vec::with_capacity(numerator.len());
//     for i in 0..numerator.len() {
//         if denominator[i] == 0 {
//             tracing::error!("attempted to divide by 0 in poly_div_vectors");
//             panic!();
//         }
//
//         let div_result = poly_div(numerator[i], denominator[i]);
//     }
//
//     todo!()
// }
