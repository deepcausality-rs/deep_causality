/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::surd::{MaxOrder, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_max_order_some_cdl() {
    let data = vec![Some(0.125); 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Some(2)).unwrap(); // Only pairwise

    // Should not have synergistic terms of order > 2
    let has_order_2_synergy = result.synergistic_info().keys().any(|k| k.len() > 2);
    assert!(!has_order_2_synergy);

    // Should have unique terms for single variables
    assert!(result.mutual_info().contains_key(&vec![1]));
    assert!(result.mutual_info().contains_key(&vec![2]));
}

#[test]
fn test_max_order_max_cdl() {
    let data = vec![Some(0.125); 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap(); // Full decomposition

    // Should not have synergistic terms of order 2 for a uniform distribution
    let has_order_2_synergy = result.synergistic_info().keys().any(|k| k.len() == 2);
    assert!(!has_order_2_synergy);
}

#[test]
fn test_max_order_min_cdl() {
    let data = vec![Some(0.125); 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Min).unwrap(); // Pairwise decomposition

    // Should not have synergistic terms of order 2 for a uniform distribution
    let has_order_2_synergy = result.synergistic_info().keys().any(|k| k.len() == 2);
    assert!(!has_order_2_synergy);
}

#[test]
fn test_max_order_some_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Some(1));
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_max_order_max_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_max_order_min_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Min);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_invalid_max_order_cdl() {
    let p_raw = CausalTensor::new(vec![Some(0.125); 8], vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Some(3)); // k=3 > n_vars=2
    assert!(matches!(
        result,
        Err(CausalTensorError::InvalidParameter(_))
    ));
}

#[test]
fn test_max_order_some_some_nones_cdl() {
    let data = vec![
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
    ]; // Shape [2,2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Some(2)).unwrap(); // Only pairwise

    let has_higher_order_synergy = result.synergistic_info().keys().any(|k| k.len() > 2);
    assert!(!has_higher_order_synergy);

    assert!(result.mutual_info().contains_key(&vec![1]));
    assert!(result.mutual_info().contains_key(&vec![2]));
    assert!(result.mutual_info().contains_key(&vec![3]));
}

#[test]
fn test_max_order_max_some_nones_cdl() {
    let data = vec![
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
    ]; // Shape [2,2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap(); // Full decomposition

    let has_higher_order_synergy = result.synergistic_info().keys().any(|k| k.len() >= 2);
    assert!(has_higher_order_synergy);
}

#[test]
fn test_max_order_min_some_nones_cdl() {
    let data = vec![
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        Some(0.1),
        Some(0.1),
    ]; // Shape [2,2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Min).unwrap(); // Pairwise decomposition

    let has_order_2_synergy = result.synergistic_info().keys().any(|k| k.len() == 2);
    assert!(has_order_2_synergy);
}
