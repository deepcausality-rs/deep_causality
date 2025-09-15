/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::surd::{MaxOrder, surd_states};
use deep_causality_data_structures::{CausalTensor, CausalTensorError};

// A small tolerance for floating point comparisons.
const TOLERANCE: f64 = 1e-10;

#[test]
fn test_empty_tensor() {
    // A tensor with a 0 in its shape has zero elements.
    let p_raw = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max);
    // The surd_states function should identify this as an empty tensor.
    assert!(matches!(result, Err(CausalTensorError::EmptyTensor)));
}

#[test]
fn test_zero_probability_tensor() {
    let p_raw = CausalTensor::new(vec![0.0; 8], vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_invalid_max_order() {
    let p_raw = CausalTensor::new(vec![0.125; 8], vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Some(3)); // k=3 > n_vars=2
    assert!(matches!(
        result,
        Err(CausalTensorError::InvalidParameter(_))
    ));
}

/// Tests the example from the `surd_states` doc comment.
#[test]
fn test_doc_comment_example_full_decomposition() {
    let data = vec![
        0.1, 0.2, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.0, 0.2, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.3, 0.0, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.1, 0.1, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();

    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

    // Check that the main aggregate maps are not empty
    assert!(!result.synergistic_info().is_empty());
    assert!(!result.redundant_info().is_empty());
    assert!(!result.mutual_info().is_empty());

    // Check a specific calculated value.
    let synergy_12 = result.synergistic_info().get(&vec![1, 2]).unwrap();
    assert!(*synergy_12 > 0.0);

    let redundancy = result.redundant_info().get(&vec![1, 2]).unwrap();
    assert!(redundancy.is_finite());

    // Info leak should be non-zero
    assert!(result.info_leak() > 0.0);
}

#[test]
fn test_partial_decomposition() {
    // System with 3 source variables
    let data = vec![0.0625; 16]; // Uniform distribution
    let p_raw = CausalTensor::new(data, vec![2, 2, 2, 2]).unwrap();

    // Run with k=2
    let result = surd_states(&p_raw, MaxOrder::Min).unwrap();

    // With a uniform distribution, all information terms should be zero.
    for val in result.synergistic_info().values() {
        assert!(val.abs() < TOLERANCE);
    }
    for val in result.redundant_info().values() {
        assert!(val.abs() < TOLERANCE);
    }

    // Crucially, check that no terms of order > 2 were computed.
    let has_order_3_synergy = result.synergistic_info().keys().any(|k| k.len() > 2);
    assert!(!has_order_3_synergy);
}

/// Test with a deterministic XOR-like distribution to check synergistic states.
#[test]
fn test_deterministic_synergy_case() {
    let data = vec![
        0.25, 0.0, // T=0, S1=0
        0.0, 0.25, // T=0, S1=1
        0.0, 0.25, // T=1, S1=0
        0.25, 0.0, // T=1, S1=1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

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

/// Test with a distribution where T=S1 to check unique states.
#[test]
fn test_unique_information_case() {
    // T = S1, S2 is independent noise.
    // P(T=0, S1=0, S2=0) = 0.25, P(T=0, S1=0, S2=1) = 0.25 -> P(T=0,S1=0)=0.5
    // P(T=1, S1=1, S2=0) = 0.25, P(T=1, S1=1, S2=1) = 0.25 -> P(T=1,S1=1)=0.5
    // All other joint probabilities are 0.
    let data = vec![
        0.25, 0.25, // T=0, S1=0
        0.0, 0.0, // T=0, S1=1
        0.0, 0.0, // T=1, S1=0
        0.25, 0.25, // T=1, S1=1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

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

/// Test with a distribution where S1 and S2 provide redundant info about T.
#[test]
fn test_redundant_information_case() {
    // T = S1, and S2 is a noisy copy of S1.
    // This creates redundancy between S1 and S2.
    let data = vec![
        0.45, 0.05, 0.0, 0.0, // T=0, S1=0, S2=0/1 and T=0, S1=1, S2=0/1
        0.0, 0.0, 0.05, 0.45, // T=1, S1=0, S2=0/1 and T=1, S1=1, S2=0/1
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

    // Check that the redundant states map is now populated.
    assert!(!result.causal_redundant_states().is_empty());
    // Also check that non-causal redundant states are populated, covering the final loop.
    assert!(!result.non_causal_redundant_states().is_empty());
}

/// Test with a distribution where T, S1, S2 are all independent.
#[test]
fn test_independent_case_full_leak() {
    // P(T), P(S1), P(S2) are all uniform (0.5/0.5)
    let data = vec![0.125; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

    // Info leak should be 1.0
    assert!((result.info_leak() - 1.0).abs() < TOLERANCE);

    // All information terms should be 0
    for val in result.mutual_info().values() {
        assert!(val.abs() < TOLERANCE);
    }
    for val in result.redundant_info().values() {
        assert!(val.abs() < TOLERANCE);
    }
    for val in result.synergistic_info().values() {
        assert!(val.abs() < TOLERANCE);
    }
}
