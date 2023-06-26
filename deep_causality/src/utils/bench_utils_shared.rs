// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

// Generates a fixed sized array with sample data
pub fn generate_sample_data<const N: usize>()
    -> [f64; N]
{
    [0.99; N]
}

