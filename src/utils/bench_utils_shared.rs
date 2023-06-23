/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

// Generates a fixed sized array with sample data
pub fn generate_sample_data<const N: usize>(
    k: usize
)
    -> [f64; N]
{
    let mut data = [0.0; N];
    for i in 0..=k {
        data[i] += 0.99;
    }

    data
}

