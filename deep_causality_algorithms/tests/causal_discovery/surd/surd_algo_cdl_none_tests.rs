/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::surd::{MaxOrder, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_mi_calculation_with_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_causal_state_maps_with_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_single_target_state_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_calculate_state_slice_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_mi_calculation_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_causal_state_maps_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_single_target_state_some_nones_cdl() {
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
    dbg!(&res);
    let result = res.unwrap();
    assert!(!result.mutual_info().is_empty());
}

#[test]
fn test_calculate_state_slice_some_nones_cdl() {
    let data = vec![
        Some(0.1),
        Some(0.1), // T=0, S1=0, S2=0
        None,
        Some(0.1), // T=0, S1=1, S2=0
        Some(0.1),
        None, // T=1, S1=0, S2=0
        Some(0.1),
        Some(0.1), // T=1, S1=1, S2=0
    ]; // Shape [2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).expect("Failed to build tensor");
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).expect(
        "Failed to run              │
 │        surd",
    );
    assert!(result.causal_unique_states().is_empty());
}
