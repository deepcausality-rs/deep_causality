/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Activity, AmountOfSubstance, HalfLife, PhysicsErrorEnum};

// =============================================================================
// AmountOfSubstance Tests
// =============================================================================

#[test]
fn test_amount_of_substance_new_valid() {
    let amount = AmountOfSubstance::new(1.0);
    assert!(amount.is_ok());
    assert!((amount.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_new_negative_error() {
    let amount = AmountOfSubstance::new(-1.0);
    assert!(amount.is_err());
    match &amount.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("AmountOfSubstance") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_amount_of_substance_new_unchecked() {
    let amount = AmountOfSubstance::new_unchecked(5.0);
    assert!((amount.value() - 5.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_into_f64() {
    let amount = AmountOfSubstance::new(2.5).unwrap();
    let val: f64 = amount.into();
    assert!((val - 2.5).abs() < 1e-10);
}

// =============================================================================
// HalfLife Tests
// =============================================================================

#[test]
fn test_half_life_new_valid() {
    let hl = HalfLife::new(5730.0); // C-14
    assert!(hl.is_ok());
}

#[test]
fn test_half_life_new_zero() {
    let hl = HalfLife::new(0.0);
    assert!(hl.is_ok());
}

#[test]
fn test_half_life_new_negative_error() {
    let hl = HalfLife::new(-100.0);
    assert!(hl.is_err());
}

#[test]
fn test_half_life_new_unchecked() {
    let hl = HalfLife::new_unchecked(1600.0); // Ra-226
    assert!((hl.value() - 1600.0).abs() < 1e-10);
}

#[test]
fn test_half_life_from_f64() {
    let hl = HalfLife::new(123.0).unwrap();
    let val: f64 = hl.into();
    assert!((val - 123.0).abs() < 1e-10);
}

// =============================================================================
// Activity Tests
// =============================================================================

#[test]
fn test_activity_new_valid() {
    let activity = Activity::new(3.7e10); // 1 Curie in Becquerels
    assert!(activity.is_ok());
}

#[test]
fn test_activity_new_negative_error() {
    let activity = Activity::new(-1.0);
    assert!(activity.is_err());
}

#[test]
fn test_activity_new_unchecked() {
    let activity = Activity::new_unchecked(1e6);
    assert!((activity.value() - 1e6).abs() < 1.0);
}

#[test]
fn test_activity_from_f64() {
    let activity = Activity::new(500.0).unwrap();
    let val: f64 = activity.into();
    assert!((val - 500.0).abs() < 1e-10);
}
