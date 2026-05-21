/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, Stiffness, Stress};

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_stress_new_valid() {
    let stress = Stress::<f64>::new(100e6);
    assert!(stress.is_ok());
    assert!((stress.unwrap().value() - 100e6).abs() < 1.0);
}

#[test]
fn test_stress_new_negative() {
    // Stress can be negative (compressive)
    let stress = Stress::<f64>::new(-50e6);
    assert!(stress.is_ok());
}

#[test]
fn test_stress_into_f64() {
    let stress = Stress::<f64>::new(200e6).unwrap();
    let val: f64 = stress.into();
    assert!((val - 200e6).abs() < 1.0);
}

#[test]
fn test_stress_default() {
    let stress = Stress::<f64>::default();
    assert!((stress.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// Stiffness Tests
// =============================================================================

#[test]
fn test_stiffness_new_valid() {
    let stiff = Stiffness::<f64>::new(200e9); // Steel Young's modulus
    assert!(stiff.is_ok());
}

#[test]
fn test_stiffness_new_negative_error() {
    let stiff = Stiffness::<f64>::new(-1.0);
    assert!(stiff.is_err());
    match &stiff.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Stiffness") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_stiffness_into_f64() {
    let stiff = Stiffness::<f64>::new(70e9).unwrap(); // Aluminum
    let val: f64 = stiff.into();
    assert!((val - 70e9).abs() < 1.0);
}

use deep_causality_physics::{StiffnessTensor, Strain, StressTensor};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Stress / Stiffness traits & defaults
// =============================================================================

#[test]
fn test_stress_traits() {
    let s = Stress::<f64>::new(1.0).unwrap();
    assert_eq!(s, s.clone());
    assert!(s < Stress::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", s);
}

#[test]
fn test_stiffness_default() {
    let s: Stiffness<f64> = Stiffness::default();
    assert_eq!(s.value(), 0.0);
}

#[test]
fn test_stiffness_traits() {
    let s = Stiffness::<f64>::new(1.0).unwrap();
    assert_eq!(s, s.clone());
    assert!(s < Stiffness::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", s);
}

// =============================================================================
// Tensor wrappers: Strain / StiffnessTensor / StressTensor
// =============================================================================

#[test]
fn test_strain_new_and_inner() {
    let t: CausalTensor<f64> = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let strain = Strain::<f64>::new(t.clone());
    assert_eq!(strain.inner().shape(), t.shape());
}

#[test]
fn test_strain_into_inner() {
    let t: CausalTensor<f64> = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let strain = Strain::<f64>::new(t.clone());
    let inner = strain.into_inner();
    assert_eq!(inner.shape(), t.shape());
}

#[test]
fn test_strain_clone_debug() {
    let t: CausalTensor<f64> = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let s = Strain::<f64>::new(t);
    let _ = s.clone();
    let _ = format!("{:?}", s);
}

#[test]
fn test_stiffness_tensor_new_inner_into() {
    let t: CausalTensor<f64> = CausalTensor::new(vec![0.0; 81], vec![3, 3, 3, 3]).unwrap();
    let st = StiffnessTensor::<f64>::new(t.clone());
    assert_eq!(st.inner().shape(), t.shape());
    let inner = st.clone().into_inner();
    assert_eq!(inner.shape(), t.shape());
    let _ = format!("{:?}", st);
}

#[test]
fn test_stress_tensor_new_inner_into() {
    let t: CausalTensor<f64> = CausalTensor::new(vec![0.0; 9], vec![3, 3]).unwrap();
    let st = StressTensor::<f64>::new(t.clone());
    assert_eq!(st.inner().shape(), t.shape());
    let inner = st.clone().into_inner();
    assert_eq!(inner.shape(), t.shape());
    let _ = format!("{:?}", st);
}
