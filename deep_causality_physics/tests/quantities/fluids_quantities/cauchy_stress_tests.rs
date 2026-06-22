/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{CauchyStress, PhysicsErrorEnum};

// =============================================================================
// CauchyStress — symmetric
// =============================================================================

#[test]
fn test_cauchy_stress_new_valid_symmetric() {
    let sigma = [[100.0, 5.0, 3.0], [5.0, 200.0, 7.0], [3.0, 7.0, 300.0]];
    let t = CauchyStress::<f64>::new(sigma).unwrap();
    assert_eq!(t.value(), &sigma);
}

#[test]
fn test_cauchy_stress_rejects_asymmetric() {
    let sigma = [[100.0, 5.0, 3.0], [99.0, 200.0, 7.0], [3.0, 7.0, 300.0]];
    let r = CauchyStress::<f64>::new(sigma);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_cauchy_stress_rejects_non_finite() {
    let sigma = [[0.0, 0.0, 0.0], [0.0, f64::NAN, 0.0], [0.0, 0.0, 0.0]];
    assert!(CauchyStress::<f64>::new(sigma).is_err());
}

#[test]
fn test_cauchy_stress_default_is_zero() {
    assert_eq!(CauchyStress::<f64>::default().into_inner(), [[0.0; 3]; 3]);
}

#[test]
fn test_cauchy_stress_new_unchecked_bypasses_check() {
    let sigma = [[1.0, 2.0, 3.0], [9.0, 5.0, 6.0], [3.0, 6.0, 9.0]];
    let t = CauchyStress::<f64>::new_unchecked(sigma);
    assert_eq!(t.into_inner(), sigma);
}

#[test]
fn test_cauchy_stress_from_self_to_raw() {
    let sigma = [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
    let t = CauchyStress::<f64>::new(sigma).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, sigma);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_cauchy_stress_traits() {
    let sigma = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = CauchyStress::<f64>::new(sigma).unwrap();
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
fn test_cauchy_stress_into_raw_matrix() {
    let m = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let s = CauchyStress::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = s.into();
    assert_eq!(raw, m);
}
