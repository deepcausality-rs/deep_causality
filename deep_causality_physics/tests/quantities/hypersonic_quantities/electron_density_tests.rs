/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{ElectronDensity, PhysicsErrorEnum};

#[test]
fn test_electron_density_valid() {
    let n = ElectronDensity::<f64>::new(1.0e19).unwrap();
    assert_eq!(n.value(), 1.0e19);
    let zero = ElectronDensity::<f64>::new(0.0).unwrap();
    assert_eq!(zero.value(), 0.0);
}

#[test]
fn test_electron_density_rejects_negative() {
    assert!(
        matches!(
            ElectronDensity::<f64>::new(-1.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_electron_density_rejects_nonfinite() {
    assert!(
        matches!(
            ElectronDensity::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            ElectronDensity::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_electron_density_new_unchecked() {
    let n = ElectronDensity::<f64>::new_unchecked(5.0e18);
    assert_eq!(n.value(), 5.0e18);
}

#[test]
fn test_electron_density_default() {
    let n: ElectronDensity<f64> = Default::default();
    assert_eq!(n.value(), 0.0);
}

#[test]
fn test_electron_density_into_f64() {
    let n = ElectronDensity::<f64>::new(2.0e17).unwrap();
    let v: f64 = n.into();
    assert_eq!(v, 2.0e17);
}
