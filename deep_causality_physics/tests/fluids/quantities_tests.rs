/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Density, PhysicsErrorEnum, Pressure, Viscosity};

// =============================================================================
// Pressure Tests
// =============================================================================

#[test]
fn test_pressure_new_valid() {
    let pressure = Pressure::new(101325.0);
    assert!(pressure.is_ok());
    assert!((pressure.unwrap().value() - 101325.0).abs() < 1e-10);
}

#[test]
fn test_pressure_new_zero() {
    let pressure = Pressure::new(0.0);
    assert!(pressure.is_ok());
}

#[test]
fn test_pressure_new_negative_error() {
    let pressure = Pressure::new(-1.0);
    assert!(pressure.is_err());
    match &pressure.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Pressure") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_pressure_new_unchecked() {
    let pressure = Pressure::new_unchecked(50000.0);
    assert!((pressure.value() - 50000.0).abs() < 1e-10);
}

#[test]
fn test_pressure_into_f64() {
    let pressure = Pressure::new(1000.0).unwrap();
    let val: f64 = pressure.into();
    assert!((val - 1000.0).abs() < 1e-10);
}

// =============================================================================
// Density Tests
// =============================================================================

#[test]
fn test_density_new_valid() {
    let density = Density::new(1000.0); // water
    assert!(density.is_ok());
    assert!((density.unwrap().value() - 1000.0).abs() < 1e-10);
}

#[test]
fn test_density_new_negative_error() {
    let density = Density::new(-1.0);
    assert!(density.is_err());
}

#[test]
fn test_density_new_unchecked() {
    let density = Density::new_unchecked(7874.0); // iron
    assert!((density.value() - 7874.0).abs() < 1e-10);
}

#[test]
fn test_density_into_f64() {
    let density = Density::new(100.0).unwrap();
    let val: f64 = density.into();
    assert!((val - 100.0).abs() < 1e-10);
}

// =============================================================================
// Viscosity Tests
// =============================================================================

#[test]
fn test_viscosity_new_valid() {
    let visc = Viscosity::new(0.001); // water at 20°C
    assert!(visc.is_ok());
}

#[test]
fn test_viscosity_new_negative_error() {
    let visc = Viscosity::new(-0.5);
    assert!(visc.is_err());
}

#[test]
fn test_viscosity_new_unchecked() {
    let visc = Viscosity::new_unchecked(1.0);
    assert!((visc.value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_viscosity_into_f64() {
    let visc = Viscosity::new(0.005).unwrap();
    let val: f64 = visc.into();
    assert!((val - 0.005).abs() < 1e-10);
}
