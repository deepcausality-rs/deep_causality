/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::BodyForceDensity;

// =============================================================================
// BodyForceDensity — finiteness only
// =============================================================================

#[test]
fn test_body_force_density_new_valid() {
    let f = BodyForceDensity::<f64>::new([0.0, 0.0, -9810.0]).unwrap();
    assert_eq!(f.value(), &[0.0, 0.0, -9810.0]);
}

#[test]
fn test_body_force_density_rejects_non_finite() {
    assert!(BodyForceDensity::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(BodyForceDensity::<f64>::new([0.0, f64::NEG_INFINITY, 0.0]).is_err());
}

#[test]
fn test_body_force_density_default() {
    assert_eq!(BodyForceDensity::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_body_force_density_new_unchecked() {
    let f = BodyForceDensity::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(f.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_body_force_density_traits() {
    let a = BodyForceDensity::<f64>::new([0.1, 0.2, 0.3]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_body_force_density_into_raw_array() {
    let f = BodyForceDensity::<f64>::new([1.5, 2.5, 3.5]).unwrap();
    let raw: [f64; 3] = f.into();
    assert_eq!(raw, [1.5, 2.5, 3.5]);
}
