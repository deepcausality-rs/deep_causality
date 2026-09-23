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
    // Stress can be negative (compressive), and the sign must survive construction.
    let stress = Stress::<f64>::new(-50e6).unwrap();
    assert!(
        (stress.value() + 50e6).abs() < 1.0,
        "stress = {}",
        stress.value()
    );
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
    let stiff = Stiffness::<f64>::new(200e9).unwrap(); // Steel Young's modulus
    assert!((stiff.value() - 200e9).abs() < 1.0, "E = {}", stiff.value());
}

#[test]
fn test_stiffness_new_negative_error() {
    let stiff = Stiffness::<f64>::new(-1.0);
    match &stiff.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Stiffness") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_stiffness_new_nan_error() {
    // materials/mod.rs:58-61 — explicit finiteness guard (NaN < 0 is false).
    let stiff = Stiffness::<f64>::new(f64::NAN);
    match &stiff.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("finite"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_stiffness_new_infinity_error() {
    let stiff = Stiffness::<f64>::new(f64::INFINITY);
    match &stiff.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("finite"));
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
    // `assert_eq!(s, s.clone())` is reflexive and holds for a `PartialEq` that always returns
    // true. The inequality is what discriminates, and comparing the two `Debug` renderings is
    // what makes `Debug` observable rather than discarded.
    let s = Stress::<f64>::new(1.0).unwrap();
    let bigger = Stress::<f64>::new(2.0).unwrap();
    assert_eq!(s, s.clone());
    assert_ne!(s, bigger);
    assert!(s < bigger);
    assert_ne!(format!("{s:?}"), format!("{bigger:?}"));
}

#[test]
fn test_stiffness_default() {
    let s: Stiffness<f64> = Stiffness::default();
    assert_eq!(s.value(), 0.0);
}

#[test]
fn test_stiffness_traits() {
    let s = Stiffness::<f64>::new(1.0).unwrap();
    let bigger = Stiffness::<f64>::new(2.0).unwrap();
    assert_eq!(s, s.clone());
    assert_ne!(s, bigger);
    assert!(s < bigger);
    assert_ne!(format!("{s:?}"), format!("{bigger:?}"));
}

// =============================================================================
// Tensor wrappers: Strain / StiffnessTensor / StressTensor
// =============================================================================

/// Nine distinct entries, so a wrapper that reordered, zeroed or truncated the data cannot
/// look the same as one that carried it through. A uniform `vec![1.0; 9]` can.
fn distinct_3x3() -> CausalTensor<f64> {
    CausalTensor::new((1..=9).map(|i| i as f64 * 0.5).collect(), vec![3, 3]).unwrap()
}

#[test]
fn test_strain_new_and_inner() {
    let t = distinct_3x3();
    let strain = Strain::<f64>::new(t.clone());
    assert_eq!(strain.inner().shape(), t.shape());
    assert_eq!(
        strain.inner().as_slice(),
        t.as_slice(),
        "the wrapper must carry the data, not merely the shape"
    );
}

#[test]
fn test_strain_into_inner() {
    let t = distinct_3x3();
    let inner = Strain::<f64>::new(t.clone()).into_inner();
    assert_eq!(inner.shape(), t.shape());
    assert_eq!(inner.as_slice(), t.as_slice());
}

#[test]
fn test_strain_clone_is_equal_and_debug_names_the_type() {
    // This test used to clone and format, discarding both, so it asserted nothing at all.
    let s = Strain::<f64>::new(distinct_3x3());
    let cloned = s.clone();
    assert_eq!(cloned.inner().as_slice(), s.inner().as_slice());

    let rendered = format!("{s:?}");
    assert!(rendered.contains("Strain"), "Debug rendering: {rendered}");
    let other = Strain::<f64>::new(CausalTensor::new(vec![0.0; 9], vec![3, 3]).unwrap());
    assert_ne!(
        rendered,
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}

#[test]
fn test_stiffness_tensor_new_inner_into() {
    let t: CausalTensor<f64> = CausalTensor::new(
        (1..=81).map(|i| i as f64 * 0.25).collect(),
        vec![3, 3, 3, 3],
    )
    .unwrap();
    let st = StiffnessTensor::<f64>::new(t.clone());
    assert_eq!(st.inner().shape(), t.shape());
    assert_eq!(st.inner().as_slice(), t.as_slice());
    let inner = st.clone().into_inner();
    assert_eq!(inner.as_slice(), t.as_slice());
    assert!(format!("{st:?}").contains("StiffnessTensor"));
}

#[test]
fn test_stress_tensor_new_inner_into() {
    let t = distinct_3x3();
    let st = StressTensor::<f64>::new(t.clone());
    assert_eq!(st.inner().shape(), t.shape());
    assert_eq!(st.inner().as_slice(), t.as_slice());
    let inner = st.clone().into_inner();
    assert_eq!(inner.as_slice(), t.as_slice());
    assert!(format!("{st:?}").contains("StressTensor"));
}
