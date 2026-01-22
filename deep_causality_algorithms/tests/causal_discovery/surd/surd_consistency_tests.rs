/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::causal_discovery::surd::{MaxOrder, surd_states};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_surd_implementation_consistency() {
    // Data from the bug report
    let data = vec![
        0.40, 0.05, // T=0, S1=0, S2=0/1
        0.02, 0.03, // T=0, S1=1, S2=0/1
        0.03, 0.02, // T=1, S1=0, S2=0/1
        0.05, 0.40, // T=1, S1=1, S2=0/1
    ];

    let shape = vec![2, 2, 2];

    // 1. Run Standard SURD (Buggy)
    let p_raw = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    let result_std = surd_states(&p_raw, MaxOrder::Max).unwrap();
    let syn_std = result_std
        .synergistic_info()
        .get(&vec![1, 2])
        .cloned()
        .unwrap_or(0.0);

    // 2. Run CDL SURD (Correct reference)
    // Convert data to Option<f64>
    let data_cdl: Vec<Option<f64>> = data.into_iter().map(Some).collect();
    let p_raw_cdl = CausalTensor::new(data_cdl, shape).unwrap();
    use deep_causality_algorithms::causal_discovery::surd::surd_states_cdl;

    let result_cdl = surd_states_cdl(&p_raw_cdl, MaxOrder::Max).unwrap();
    let syn_cdl = result_cdl
        .synergistic_info()
        .get(&vec![1, 2])
        .cloned()
        .unwrap_or(0.0);

    println!("Standard Synergy [1,2]: {}", syn_std);
    println!("CDL Synergy [1,2]:      {}", syn_cdl);

    // The bug causes these to differ.
    // Standard implementation zeros out info incorrectly, leading to specific value differences.
    // We expect this assertion to FAIL before the fix.
    assert!(
        (syn_std - syn_cdl).abs() < 1e-10,
        "Standard SURD does not match CDL reference! Bug reproduced."
    );
}
