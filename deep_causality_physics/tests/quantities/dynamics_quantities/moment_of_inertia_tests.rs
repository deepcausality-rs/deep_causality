/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::MomentOfInertia;

#[test]
fn test_moment_of_inertia_new_valid() {
    let moi = MomentOfInertia::<f64>::new(5.0);
    assert!(moi.is_ok());
}

#[test]
fn test_moment_of_inertia_new_negative_error() {
    let moi = MomentOfInertia::<f64>::new(-1.0);
    assert!(moi.is_err());
}

#[test]
fn test_moment_of_inertia_new_nan_error() {
    let moi = MomentOfInertia::<f64>::new(f64::NAN);
    assert!(moi.is_err());
}

#[test]
fn test_moment_of_inertia_new_infinity_error() {
    let moi = MomentOfInertia::<f64>::new(f64::INFINITY);
    assert!(moi.is_err());
}

#[test]
fn test_moment_of_inertia_new_unchecked() {
    let moi = MomentOfInertia::<f64>::new_unchecked(7.5);
    assert!((moi.value() - 7.5).abs() < 1e-10);
}

#[test]
fn test_moment_of_inertia_into_f64() {
    let moi = MomentOfInertia::<f64>::new(2.0).unwrap();
    let val: f64 = moi.into();
    assert!((val - 2.0).abs() < 1e-10);
}

#[test]
fn test_moment_of_inertia_default() {
    let m: MomentOfInertia<f64> = MomentOfInertia::default();
    assert_eq!(m.value(), 0.0);
}
