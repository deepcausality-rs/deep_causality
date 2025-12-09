/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, Stiffness, Stress};

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_stress_new_valid() {
    let stress = Stress::new(100e6);
    assert!(stress.is_ok());
    assert!((stress.unwrap().value() - 100e6).abs() < 1.0);
}

#[test]
fn test_stress_new_negative() {
    // Stress can be negative (compressive)
    let stress = Stress::new(-50e6);
    assert!(stress.is_ok());
}

#[test]
fn test_stress_into_f64() {
    let stress = Stress::new(200e6).unwrap();
    let val: f64 = stress.into();
    assert!((val - 200e6).abs() < 1.0);
}

#[test]
fn test_stress_default() {
    let stress = Stress::default();
    assert!((stress.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// Stiffness Tests
// =============================================================================

#[test]
fn test_stiffness_new_valid() {
    let stiff = Stiffness::new(200e9); // Steel Young's modulus
    assert!(stiff.is_ok());
}

#[test]
fn test_stiffness_new_negative_error() {
    let stiff = Stiffness::new(-1.0);
    assert!(stiff.is_err());
    match &stiff.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Stiffness") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_stiffness_into_f64() {
    let stiff = Stiffness::new(70e9).unwrap(); // Aluminum
    let val: f64 = stiff.into();
    assert!((val - 70e9).abs() < 1.0);
}
