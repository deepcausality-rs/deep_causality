/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::causal_discovery::surd::{MaxOrder, surd_states};
use deep_causality_data_structures::CausalTensor;

fn main() {
    let data_original = vec![
        0.1, 0.2, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.0, 0.2, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.3, 0.0, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.1, 0.1, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw_original = CausalTensor::new(data_original, vec![2, 2, 2]).unwrap();
    let full_result_original = surd_states(&p_raw_original, MaxOrder::Max).unwrap();
    println!("{}", &full_result_original);

    // --- Test Case 1: Low Information Leak (Strong Dependence) ---
    // Target (T) is strongly determined by Source 1 (S1) and Source 2 (S2).
    // Example: T = S1 XOR S2 (simplified for probabilities)
    // P(T=0, S1=0, S2=0) = high, P(T=0, S1=0, S2=1) = high
    // P(T=1, S1=0, S2=1) = high, P(T=1, S1=1, S2=0) = high
    // Other combinations are low.
    println!("\n--- Test Case: Low Information Leak ---");
    let data_low_leak = vec![
        0.45, 0.05, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.05, 0.45, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.05, 0.45, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.45, 0.05, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw_low_leak = CausalTensor::new(data_low_leak, vec![2, 2, 2]).unwrap();
    let result_low_leak = surd_states(&p_raw_low_leak, MaxOrder::Max).unwrap();
    println!("{}", &result_low_leak);

    // --- Test Case 2: Medium Information Leak (Partial Dependence) ---
    // Target (T) depends on Source 1 (S1), but also has some randomness.
    // S2 has little to no influence.
    println!("\n--- Test Case: Medium Information Leak ---");
    let data_medium_leak = vec![
        0.3, 0.1, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.2, 0.1, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.1, 0.2, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.1, 0.3, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw_medium_leak = CausalTensor::new(data_medium_leak, vec![2, 2, 2]).unwrap();
    let result_medium_leak = surd_states(&p_raw_medium_leak, MaxOrder::Max).unwrap();
    println!("{}", &result_medium_leak);

    // --- Test Case 3: High Information Leak (Weak Dependence) ---
    // Target (T) is mostly random, with very little dependence on S1 and S2.
    // Probabilities are roughly uniform.
    println!("\n--- Test Case: High Information Leak ---");
    let data_high_leak = vec![
        0.12, 0.13, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.13, 0.12, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.13, 0.12, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.12, 0.13, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw_high_leak = CausalTensor::new(data_high_leak, vec![2, 2, 2]).unwrap();
    let result_high_leak = surd_states(&p_raw_high_leak, MaxOrder::Max).unwrap();
    println!("{}", &result_high_leak);
}
