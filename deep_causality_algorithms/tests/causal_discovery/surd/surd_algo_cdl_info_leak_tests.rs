/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::surd::{MaxOrder, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, CausalTensorError};
const TOLERANCE: f64 = 1e-10;

#[test]
fn test_info_leak_zero_cdl() {
    // T is fully determined by S1 and S2, no Nones
    let data = vec![
        Some(0.25),
        Some(0.0),
        Some(0.0),
        Some(0.25),
        Some(0.0),
        Some(0.25),
        Some(0.25),
        Some(0.0),
    ]; // XOR-like
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak().abs() < TOLERANCE);
}

#[test]
fn test_info_leak_with_nones_cdl() {
    // T is independent of S1, but S1 has some Nones
    let data = vec![
        Some(0.25),
        Some(0.25), // T=0, S1=0, S2=0
        None,
        Some(0.25), // T=0, S1=1, S2=0
        Some(0.25),
        None, // T=1, S1=0, S2=0
        None,
        None, // T=1, S1=1, S2=0
    ]; // Shape [2,2,2]
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).expect("Failed to build tensor");
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).expect("Failed to run SURD");

    // Info leak should be high if T is independent of S1
    assert!(result.info_leak() > 0.5);
}
#[test]
fn test_info_leak_all_nones_cdl() {
    let data = vec![None; 8];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max);
    assert!(matches!(result, Err(CausalTensorError::InvalidOperation)));
}

#[test]
fn test_info_leak_partial_cdl() {
    // T is partially determined, some Nones
    let data = vec![
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak() > 0.0);
    assert!(result.info_leak() < 1.0);
}

#[test]
fn test_info_leak_zero_some_nones_cdl() {
    // T is fully determined by S1 and S2, with some Nones
    let data = vec![
        Some(0.25),
        None,
        None,
        Some(0.25),
        None,
        Some(0.25),
        Some(0.25),
        None,
    ]; // XOR-like with Nones
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak().abs() < TOLERANCE);
}

#[test]
fn test_info_leak_partial_some_nones_cdl() {
    // T is partially determined, some Nones
    let data = vec![
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
        None,
        Some(0.1),
        Some(0.1),
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak() > 0.0);
    assert!(result.info_leak() < 1.0);
}
