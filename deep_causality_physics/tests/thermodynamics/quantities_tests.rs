/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Efficiency, Entropy};
use deep_causality_physics::PhysicsErrorEnum;

// =============================================================================
// Entropy Tests
// =============================================================================

#[test]
fn test_entropy_new_valid() {
    let entropy = Entropy::new(100.0);
    assert!(entropy.is_ok());
    assert!((entropy.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_entropy_new_negative() {
    // Entropy can be negative in some contexts (relative entropy)
    let entropy = Entropy::new(-10.0);
    assert!(entropy.is_ok());
}

#[test]
fn test_entropy_default() {
    let entropy = Entropy::default();
    assert!((entropy.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_entropy_into_f64() {
    let entropy = Entropy::new_unchecked(42.0);
    let val: f64 = entropy.into();
    assert!((val - 42.0).abs() < 1e-10);
}

// =============================================================================
// Efficiency Tests
// =============================================================================

#[test]
fn test_efficiency_new_valid() {
    let eff = Efficiency::new(0.5);
    assert!(eff.is_ok());
    assert!((eff.unwrap().value() - 0.5).abs() < 1e-10);
}

#[test]
fn test_efficiency_new_zero() {
    let eff = Efficiency::new(0.0);
    assert!(eff.is_ok());
}

#[test]
fn test_efficiency_new_one() {
    let eff = Efficiency::new(1.0);
    assert!(eff.is_ok());
}

#[test]
fn test_efficiency_new_error_negative() {
    let eff = Efficiency::new(-0.1);
    assert!(eff.is_err());
    let err = eff.unwrap_err();
    assert!(matches!(&err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)));
}

#[test]
fn test_efficiency_new_error_greater_than_one() {
    let eff = Efficiency::new(1.1);
    assert!(eff.is_err());
}

#[test]
fn test_efficiency_into_f64() {
    let eff = Efficiency::new(0.75).unwrap();
    let val: f64 = eff.into();
    assert!((val - 0.75).abs() < 1e-10);
}
