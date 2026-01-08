/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::causal_discovery::surd::{MaxOrder, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

// A small tolerance for floating point comparisons.
const TOLERANCE: f64 = 1e-10;

#[test]
fn test_empty_tensor_cdl() {
    // A tensor with a 0 in its shape has zero elements.
    let p_raw = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    // The surd_states_cdl function should identify this as an empty tensor.
    assert!(matches!(result, Err(CausalTensorError::EmptyTensor)));
}

#[test]
fn test_zero_probability_tensor_cdl() {
    // All Some values are 0.0
    let p_raw = CausalTensor::new(vec![Some(0.0); 8], vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));

    // All values are None
    let p_raw_none = CausalTensor::new(vec![None; 8], vec![2, 2, 2]).unwrap();
    let result_none = surd_states_cdl(&p_raw_none, MaxOrder::Max);
    assert!(matches!(
        result_none,
        Err(CausalTensorError::InvalidOperation)
    ));

    // Mix of Some(0.0) and None
    let p_raw_mixed = CausalTensor::new(
        vec![
            Some(0.0),
            None,
            Some(0.0),
            None,
            Some(0.0),
            None,
            Some(0.0),
            None,
        ],
        vec![2, 2, 2],
    )
    .unwrap();
    let result_mixed = surd_states_cdl(&p_raw_mixed, MaxOrder::Max);
    assert!(matches!(
        result_mixed,
        Err(CausalTensorError::InvalidOperation)
    ));
}

/// Tests the example from the `surd_states_cdl` doc comment with None values.
#[test]
fn test_doc_comment_example_full_decomposition_cdl() {
    let data = vec![
        Some(0.1),
        Some(0.2), // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        None,
        Some(0.2), // P(T=0, S1=1, S2=0) is missing, P(T=0, S1=1, S2=1)
        Some(0.3),
        None, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1) is missing
        Some(0.1),
        Some(0.1), // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();

    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();

    // Check that the main aggregate maps are not empty
    assert!(!result.synergistic_info().is_empty());
    assert!(!result.redundant_info().is_empty());
    assert!(!result.mutual_info().is_empty());

    // Check a specific calculated value.
    // The exact values will differ from the non-cdl version due to None handling.
    // We expect some non-zero values.
    let synergy_12 = result.synergistic_info().get(&vec![1, 2]).unwrap();
    assert!(*synergy_12 > 0.0);

    let redundancy = result.redundant_info().get(&vec![1, 2]).unwrap();
    assert!(redundancy.is_finite());

    // Info leak should be non-zero
    assert!(result.info_leak() > 0.0);
}

#[test]
fn test_partial_decomposition_cdl() {
    // System with 3 source variables, with some None values
    let data = vec![
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        None,
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        None,
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
        Some(0.0625),
    ]; // Mostly uniform distribution with some None
    let p_raw = CausalTensor::new(data, vec![2, 2, 2, 2]).unwrap();

    // Run with k=2
    let result = surd_states_cdl(&p_raw, MaxOrder::Min).unwrap();

    // With a distribution that is mostly uniform but with Nones,
    // information terms should still be close to zero if Nones are handled as "no impact".
    for val in result.synergistic_info().values() {
        assert!(val.is_finite());
    }
    for val in result.redundant_info().values() {
        assert!(val.is_finite());
    }

    // Crucially, check that no terms of order > 2 were computed.
    let has_order_3_synergy = result.synergistic_info().keys().any(|k| k.len() > 2);
    assert!(!has_order_3_synergy);
}

/// Test with a deterministic XOR-like distribution with None values to check synergistic states.
#[test]
fn test_deterministic_synergy_case_cdl() {
    let data = vec![
        Some(0.25),
        None, // T=0, S1=0
        None,
        Some(0.25), // T=0, S1=1
        None,
        Some(0.25), // T=1, S1=0
        Some(0.25),
        None, // T=1, S1=1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();

    // Info leak should be 0
    assert!(result.info_leak().abs() < TOLERANCE);

    // In a pure XOR, redundancy is 0. If the key is not found, it's implicitly zero.
    let redundancy = result
        .redundant_info()
        .get(&vec![1, 2])
        .copied()
        .unwrap_or(0.0);
    assert!(redundancy.abs() < TOLERANCE);

    // All information is synergistic. I(T;S1,S2) = 1 bit.
    let synergy = result.synergistic_info().get(&vec![1, 2]).unwrap();
    assert!((synergy - 1.0).abs() < TOLERANCE);

    // Check that the synergistic states map is populated and others are empty.
    assert!(!result.causal_synergistic_states().is_empty());
    assert!(result.causal_unique_states().is_empty());
    assert!(result.causal_redundant_states().is_empty());
}

/// Test with a distribution where T=S1 to check unique states, with None values.
#[test]
fn test_unique_information_case_cdl() {
    // T = S1, S2 is independent noise.
    // P(T=0, S1=0, S2=0) = 0.25, P(T=0, S1=0, S2=1) = 0.25 -> P(T=0,S1=0)=0.5
    // P(T=1, S1=1, S2=0) = 0.25, P(T=1, S1=1, S2=1) = 0.25 -> P(T=1,S1=1)=0.5
    // All other joint probabilities are 0.
    let data = vec![
        Some(0.25),
        Some(0.25), // T=0, S1=0
        None,
        None, // T=0, S1=1
        None,
        None, // T=1, S1=0
        Some(0.25),
        Some(0.25), // T=1, S1=1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();

    // Info leak should be 0, as T is fully determined by S1.
    assert!(result.info_leak().abs() < TOLERANCE);

    // Check MI: I(T;S1) = 1 bit, I(T;S2) = 0 bits, I(T;S1,S2) = 1 bit.
    let mi_s1 = result.mutual_info().get(&vec![1]).unwrap();
    let mi_s2 = result.mutual_info().get(&vec![2]).unwrap();
    assert!((mi_s1 - 1.0).abs() < TOLERANCE);
    assert!(mi_s2.abs() < TOLERANCE);

    // Check that the unique states map for S1 is populated.
    assert!(!result.causal_unique_states().is_empty());
    assert!(result.causal_unique_states().contains_key(&vec![1]));

    // Check that other state maps are empty.
    assert!(result.causal_synergistic_states().is_empty());
    assert!(result.causal_redundant_states().is_empty());
}

/// Test with a distribution where S1 and S2 provide redundant info about T, with None values.
#[test]
fn test_redundant_information_case_cdl() {
    // T = S1, and S2 is a noisy copy of S1.
    // This creates redundancy between S1 and S2.
    let data = vec![
        Some(0.45),
        Some(0.05),
        None,
        None, // T=0, S1=0, S2=0/1 and T=0, S1=1, S2=0/1
        None,
        None,
        Some(0.05),
        Some(0.45), // T=1, S1=0, S2=0/1 and T=1, S1=1, S2=0/1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();

    // Check that the redundant states map is now populated.
    assert!(!result.causal_redundant_states().is_empty());
    // Also check that non-causal redundant states are populated, covering the final loop.
    assert!(!result.non_causal_redundant_states().is_empty());
}

#[test]
fn test_all_none_input_cdl() {
    let p_raw = CausalTensor::new(vec![None; 8], vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_some_none_marginalization_cdl() {
    // Test case where marginalization results in None
    let data = vec![Some(0.1), None, None, Some(0.1)]; // Shape [2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2]).unwrap(); // T, S1

    // This will cause p_s (marginal of S1) to have Some and None
    // p_s = [Some(0.1), Some(0.1)]
    // p_as = [Some(0.1), None, None, Some(0.1)]
    // p_j = [Some(0.1), Some(0.1)]
    // p_s_a = [Some(1.0), None, None, Some(1.0)]
    // p_a_s = [Some(1.0), None, None, Some(1.0)]

    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak().abs() < TOLERANCE); // T is fully determined by S1
    let mi_s1 = result.mutual_info().get(&vec![1]).unwrap();
    assert!((mi_s1 - 1.0).abs() < TOLERANCE);
}

#[test]
fn test_analyze_single_target_state_none_handling_cdl() {
    // Test case to ensure analyze_single_target_state_cdl handles Nones correctly
    // Specifically, the sorting and info calculation.
    let data = vec![
        Some(0.1),
        Some(0.0), // T=0, S1=0, S2=0
        None,
        Some(0.1), // T=0, S1=1, S2=0
        Some(0.1),
        None, // T=1, S1=0, S2=0
        Some(0.1),
        Some(0.1), // T=1, S1=1, S2=0
    ]; // Shape [2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).expect("Failed to build Tensor");
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).expect("Failed to to run SURD");

    // Just ensure it runs without panic and produces some results
    assert!(!result.mutual_info().is_empty());
}

#[test]
fn test_calculate_state_slice_none_handling_cdl() {
    // Test various None combinations in calculate_state_slice_cdl
    let data = vec![
        Some(0.1),
        Some(0.1), // T=0, S1=0, S2=0
        None,
        Some(0.1), // T=0, S1=1, S2=0
        Some(0.1),
        None, // T=1, S1=0, S2=0
        Some(0.1),
        Some(0.1), // T=1, S1=1, S2=0
    ]; // Shape [2,2,1]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).expect("Failed to build tensor");

    let res = surd_states_cdl(&p_raw, MaxOrder::Max);
    let result = res.unwrap();

    // Just ensure it runs without panic and produces some results
    assert!(result.causal_unique_states().is_empty());
}
