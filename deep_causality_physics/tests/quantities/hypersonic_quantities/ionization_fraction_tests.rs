/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{IonizationFraction, PhysicsErrorEnum};

#[test]
fn test_ionization_fraction_valid() {
    assert_eq!(IonizationFraction::<f64>::new(0.0).unwrap().value(), 0.0);
    assert_eq!(IonizationFraction::<f64>::new(0.5).unwrap().value(), 0.5);
    assert_eq!(IonizationFraction::<f64>::new(1.0).unwrap().value(), 1.0);
}

#[test]
fn test_ionization_fraction_rejects_out_of_range() {
    assert!(
        matches!(
            IonizationFraction::<f64>::new(-0.01).unwrap_err().0,
            PhysicsErrorEnum::NormalizationError { .. }
        ),
        "expected a NormalizationError refusal"
    );
    assert!(
        matches!(
            IonizationFraction::<f64>::new(1.01).unwrap_err().0,
            PhysicsErrorEnum::NormalizationError { .. }
        ),
        "expected a NormalizationError refusal"
    );
}

#[test]
fn test_ionization_fraction_rejects_nonfinite() {
    assert!(
        matches!(
            IonizationFraction::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::NormalizationError { .. }
        ),
        "expected a NormalizationError refusal"
    );
    assert!(
        matches!(
            IonizationFraction::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::NormalizationError { .. }
        ),
        "expected a NormalizationError refusal"
    );
}

#[test]
fn test_ionization_fraction_new_unchecked() {
    let a = IonizationFraction::<f64>::new_unchecked(0.25);
    assert_eq!(a.value(), 0.25);
}

#[test]
fn test_ionization_fraction_default() {
    let a: IonizationFraction<f64> = Default::default();
    assert_eq!(a.value(), 0.0);
}

#[test]
fn test_ionization_fraction_into_f64() {
    let a = IonizationFraction::<f64>::new(0.3).unwrap();
    let v: f64 = a.into();
    assert_eq!(v, 0.3);
}
