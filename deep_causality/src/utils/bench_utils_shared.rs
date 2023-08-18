// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

// Generates a fixed sized array with sample data
pub fn generate_sample_data<const N: usize>()
    -> [f64; N]
{
    [0.99; N]
}

