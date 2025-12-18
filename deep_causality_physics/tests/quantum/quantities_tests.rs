/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhaseAngle, Probability};
use std::f64::consts::PI;
// =============================================================================
// Probability Tests
// =============================================================================

#[test]
fn test_probability_new_valid() {
    let prob = Probability::new(0.5);
    assert!(prob.is_ok());
    assert!((prob.unwrap().value() - 0.5).abs() < 1e-10);
}

#[test]
fn test_probability_new_zero() {
    let prob = Probability::new(0.0);
    assert!(prob.is_ok());
    assert!((prob.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_probability_new_one() {
    let prob = Probability::new(1.0);
    assert!(prob.is_ok());
    assert!((prob.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_probability_new_error_negative() {
    let prob = Probability::new(-0.1);
    assert!(prob.is_err());
}

#[test]
fn test_probability_new_error_greater_than_one() {
    let prob = Probability::new(1.1);
    assert!(prob.is_err());
}

#[test]
fn test_probability_new_nan_error() {
    let prob = Probability::new(f64::NAN);
    assert!(prob.is_err());
}

#[test]
fn test_probability_new_infinity_error() {
    let prob = Probability::new(f64::INFINITY);
    assert!(prob.is_err());
}

#[test]
fn test_probability_new_unchecked() {
    let prob = Probability::new_unchecked(1.5);
    // Unchecked allows invalid values
    assert!((prob.value() - 1.5).abs() < 1e-10);
}

#[test]
fn test_probability_into_f64() {
    let prob = Probability::new(0.75).unwrap();
    let val: f64 = prob.into();
    assert!((val - 0.75).abs() < 1e-10);
}

// =============================================================================
// PhaseAngle Tests
// =============================================================================

#[test]
fn test_phase_angle_new_valid() {
    let angle = PhaseAngle::new(PI);
    assert!(angle.is_ok());
    assert!((angle.unwrap().value() - PI).abs() < 1e-10);
}

#[test]
fn test_phase_angle_new_nan_error() {
    let angle = PhaseAngle::new(f64::NAN);
    assert!(angle.is_err());
}

#[test]
fn test_phase_angle_new_infinity_error() {
    let angle = PhaseAngle::new(f64::INFINITY);
    assert!(angle.is_err());
}

#[test]
fn test_phase_angle_default() {
    let angle = PhaseAngle::default();
    assert!((angle.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_phase_angle_into_f64() {
    let angle = PhaseAngle::new_unchecked(1.23);
    let val: f64 = angle.into();
    assert!((val - 1.23).abs() < 1e-10);
}
