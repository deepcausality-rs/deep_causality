/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Mass, PhysicsErrorEnum};

#[test]
fn test_mass_new_valid() {
    let mass = Mass::<f64>::new(10.0);
    assert!(mass.is_ok());
    assert!((mass.unwrap().value() - 10.0).abs() < 1e-10);
}

#[test]
fn test_mass_new_zero() {
    let mass = Mass::<f64>::new(0.0);
    assert!(mass.is_ok());
    assert!((mass.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_mass_new_negative_error() {
    let mass = Mass::<f64>::new(-1.0);
    assert!(mass.is_err());
    match &mass.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_mass_new_nan_error() {
    let mass = Mass::<f64>::new(f64::NAN);
    assert!(mass.is_err());
    match &mass.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_mass_new_infinity_error() {
    let mass = Mass::<f64>::new(f64::INFINITY);
    assert!(mass.is_err());
    match &mass.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_mass_new_unchecked() {
    let mass = Mass::<f64>::new_unchecked(42.0);
    assert!((mass.value() - 42.0).abs() < 1e-10);
}

#[test]
fn test_mass_into_f64() {
    let mass = Mass::<f64>::new(5.0).unwrap();
    let val: f64 = mass.into();
    assert!((val - 5.0).abs() < 1e-10);
}

#[test]
fn test_mass_default() {
    let mass = Mass::<f64>::default();
    assert!((mass.value() - 0.0).abs() < 1e-10);
}
