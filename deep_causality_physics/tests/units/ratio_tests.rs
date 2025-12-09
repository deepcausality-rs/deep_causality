/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Ratio;

#[test]
fn test_ratio_new_valid() {
    let r = Ratio::new(0.5);
    assert!(r.is_ok());
    assert!((r.unwrap().value() - 0.5).abs() < 1e-10);
}

#[test]
fn test_ratio_new_zero() {
    let r = Ratio::new(0.0);
    assert!(r.is_ok());
    assert!((r.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_ratio_new_one() {
    let r = Ratio::new(1.0);
    assert!(r.is_ok());
    assert!((r.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_ratio_new_negative() {
    // Ratio can be negative (e.g., for relative comparisons)
    let r = Ratio::new(-0.5);
    assert!(r.is_ok());
    assert!((r.unwrap().value() - (-0.5)).abs() < 1e-10);
}

#[test]
fn test_ratio_new_greater_than_one() {
    // Ratio can be > 1 (e.g., amplification factors)
    let r = Ratio::new(2.5);
    assert!(r.is_ok());
    assert!((r.unwrap().value() - 2.5).abs() < 1e-10);
}

#[test]
fn test_ratio_default() {
    let r = Ratio::default();
    assert!((r.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_ratio_into_f64() {
    let r = Ratio::new(0.75).unwrap();
    let val: f64 = r.into();
    assert!((val - 0.75).abs() < 1e-10);
}
