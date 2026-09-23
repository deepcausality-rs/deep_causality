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
    let h = SpecificEnthalpy::<f64>::new(0.0).unwrap();
    // `is_ok()` alone admitted any carried value, including a constant.
    assert!(
        (h.value() - (0.0)).abs() < 1e-10,
        "constructed value = {}",
        h.value()
    );
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
    match &h.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_specific_enthalpy_new_infinity_error() {
    assert!(
        matches!(
            SpecificEnthalpy::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            SpecificEnthalpy::<f64>::new(f64::NEG_INFINITY)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
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
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = SpecificEnthalpy::<f64>::new(2.0e5).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}
