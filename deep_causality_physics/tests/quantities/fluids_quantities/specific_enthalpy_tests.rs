/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, SpecificEnthalpy};

// =============================================================================
// SpecificEnthalpy Tests (J/kg) — signed allowed
// =============================================================================

#[test]
fn test_specific_enthalpy_new_positive() {
    let h = SpecificEnthalpy::<f64>::new(2.5e5);
    assert!(h.is_ok());
    assert!((h.unwrap().value() - 2.5e5).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_new_zero() {
    let h = SpecificEnthalpy::<f64>::new(0.0);
    assert!(h.is_ok());
}

#[test]
fn test_specific_enthalpy_new_negative_allowed() {
    // h is reference-state dependent and may be negative.
    let h = SpecificEnthalpy::<f64>::new(-1.0e4);
    assert!(h.is_ok());
    assert!((h.unwrap().value() + 1.0e4).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_new_nan_error() {
    let h = SpecificEnthalpy::<f64>::new(f64::NAN);
    assert!(h.is_err());
    match &h.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_specific_enthalpy_new_infinity_error() {
    assert!(SpecificEnthalpy::<f64>::new(f64::INFINITY).is_err());
    assert!(SpecificEnthalpy::<f64>::new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_specific_enthalpy_new_unchecked() {
    let h = SpecificEnthalpy::<f64>::new_unchecked(3.0e5);
    assert!((h.value() - 3.0e5).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_default() {
    let h: SpecificEnthalpy<f64> = SpecificEnthalpy::default();
    assert_eq!(h.value(), 0.0);
}

#[test]
fn test_specific_enthalpy_into_f64() {
    let h = SpecificEnthalpy::<f64>::new(1.0e5).unwrap();
    let v: f64 = h.into();
    assert!((v - 1.0e5).abs() < 1e-6);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_specific_enthalpy_traits() {
    let a = SpecificEnthalpy::<f64>::new(1.0e5).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert!(a < SpecificEnthalpy::<f64>::new(2.0e5).unwrap());
    let _ = format!("{:?}", a);
}
