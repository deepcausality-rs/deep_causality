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

/// Test with a deterministic XOR-like distribution.
#[test]
fn test_deterministic_case_zero_leak() {
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

    // In a pure XOR, redundancy is 0.
    let redundancy = result.redundant_info().get(&vec![1, 2]).unwrap();
    assert!(redundancy.abs() < TOLERANCE);

    // All information is synergistic. I(T;S1,S2) = 1 bit.
    let synergy = result.synergistic_info().get(&vec![1, 2]).unwrap();
    assert!((synergy - 1.0).abs() < TOLERANCE);
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
