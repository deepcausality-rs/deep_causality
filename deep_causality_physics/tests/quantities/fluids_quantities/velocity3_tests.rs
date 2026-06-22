/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, Velocity3};

// =============================================================================
// Velocity3 — finiteness-only invariant
// =============================================================================

#[test]
fn test_velocity3_new_valid() {
    let u = Velocity3::<f64>::new([1.0, -2.0, 3.5]).unwrap();
    assert_eq!(u.value(), &[1.0, -2.0, 3.5]);
}

#[test]
fn test_velocity3_new_rejects_nan() {
    assert!(Velocity3::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, f64::NAN, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, 0.0, f64::NAN]).is_err());
}

#[test]
fn test_velocity3_new_rejects_infinity() {
    assert!(Velocity3::<f64>::new([f64::INFINITY, 0.0, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, f64::NEG_INFINITY, 0.0]).is_err());
}

#[test]
fn test_velocity3_new_error_message_mentions_finite() {
    match &Velocity3::<f64>::new([f64::NAN, 0.0, 0.0]).unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_velocity3_default() {
    assert_eq!(Velocity3::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_velocity3_new_unchecked() {
    let u = Velocity3::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(u.value(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_velocity3_into_inner_consumes() {
    let u = Velocity3::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    assert_eq!(u.into_inner(), [4.0, 5.0, 6.0]);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_velocity3_traits() {
    let a = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
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
fn test_velocity3_into_raw_array() {
    let v = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let raw: [f64; 3] = v.into();
    assert_eq!(raw, [1.0, 2.0, 3.0]);
}
