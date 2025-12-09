/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Acceleration, Area, Force, Frequency, Length, Mass, MomentOfInertia, PhysicsErrorEnum, Speed,
    Torque, Volume,
};

// =============================================================================
// Mass Tests
// =============================================================================

#[test]
fn test_mass_new_valid() {
    let mass = Mass::new(10.0);
    assert!(mass.is_ok());
    assert!((mass.unwrap().value() - 10.0).abs() < 1e-10);
}

#[test]
fn test_mass_new_zero() {
    let mass = Mass::new(0.0);
    assert!(mass.is_ok());
    assert!((mass.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_mass_new_negative_error() {
    let mass = Mass::new(-1.0);
    assert!(mass.is_err());
    match &mass.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("negative") || msg.contains("Mass"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_mass_new_unchecked() {
    let mass = Mass::new_unchecked(42.0);
    assert!((mass.value() - 42.0).abs() < 1e-10);
}

#[test]
fn test_mass_into_f64() {
    let mass = Mass::new(5.0).unwrap();
    let val: f64 = mass.into();
    assert!((val - 5.0).abs() < 1e-10);
}

#[test]
fn test_mass_default() {
    let mass = Mass::default();
    assert!((mass.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// Speed Tests
// =============================================================================

#[test]
fn test_speed_new_valid() {
    let speed = Speed::new(100.0);
    assert!(speed.is_ok());
    assert!((speed.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_speed_new_zero() {
    let speed = Speed::new(0.0);
    assert!(speed.is_ok());
}

#[test]
fn test_speed_new_negative_error() {
    let speed = Speed::new(-50.0);
    assert!(speed.is_err());
}

#[test]
fn test_speed_into_f64() {
    let speed = Speed::new(299792458.0).unwrap();
    let val: f64 = speed.into();
    assert!((val - 299792458.0).abs() < 1.0);
}

#[test]
fn test_speed_new_unchecked() {
    let speed = Speed::new_unchecked(123.45);
    assert!((speed.value() - 123.45).abs() < 1e-10);
}

// =============================================================================
// Length Tests
// =============================================================================

#[test]
fn test_length_new_valid() {
    let length = Length::new(1.5e11);
    assert!(length.is_ok());
}

#[test]
fn test_length_new_negative_error() {
    let length = Length::new(-1.0);
    assert!(length.is_err());
}

#[test]
fn test_length_default() {
    let length = Length::default();
    assert!((length.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_length_new_unchecked() {
    let length = Length::new_unchecked(99.9);
    assert!((length.value() - 99.9).abs() < 1e-10);
}

#[test]
fn test_length_into_f64() {
    let length = Length::new(10.0).unwrap();
    let val: f64 = length.into();
    assert!((val - 10.0).abs() < 1e-10);
}

// =============================================================================
// Area Tests
// =============================================================================

#[test]
fn test_area_new_valid() {
    let area = Area::new(100.0);
    assert!(area.is_ok());
    assert!((area.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_area_new_negative_error() {
    let area = Area::new(-10.0);
    assert!(area.is_err());
}

#[test]
fn test_area_new_unchecked() {
    let area = Area::new_unchecked(50.5);
    assert!((area.value() - 50.5).abs() < 1e-10);
}

#[test]
fn test_area_into_f64() {
    let area = Area::new(25.0).unwrap();
    let val: f64 = area.into();
    assert!((val - 25.0).abs() < 1e-10);
}

// =============================================================================
// Volume Tests
// =============================================================================

#[test]
fn test_volume_new_valid() {
    let volume = Volume::new(1000.0);
    assert!(volume.is_ok());
}

#[test]
fn test_volume_new_negative_error() {
    let volume = Volume::new(-1.0);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_unchecked() {
    let volume = Volume::new_unchecked(123.0);
    assert!((volume.value() - 123.0).abs() < 1e-10);
}

#[test]
fn test_volume_into_f64() {
    let volume = Volume::new(100.0).unwrap();
    let val: f64 = volume.into();
    assert!((val - 100.0).abs() < 1e-10);
}

// =============================================================================
// MomentOfInertia Tests
// =============================================================================

#[test]
fn test_moment_of_inertia_new_valid() {
    let moi = MomentOfInertia::new(5.0);
    assert!(moi.is_ok());
}

#[test]
fn test_moment_of_inertia_new_negative_error() {
    let moi = MomentOfInertia::new(-1.0);
    assert!(moi.is_err());
}

#[test]
fn test_moment_of_inertia_new_unchecked() {
    let moi = MomentOfInertia::new_unchecked(7.5);
    assert!((moi.value() - 7.5).abs() < 1e-10);
}

#[test]
fn test_moment_of_inertia_into_f64() {
    let moi = MomentOfInertia::new(2.0).unwrap();
    let val: f64 = moi.into();
    assert!((val - 2.0).abs() < 1e-10);
}

// =============================================================================
// Frequency Tests
// =============================================================================

#[test]
fn test_frequency_new_valid() {
    let freq = Frequency::new(440.0); // A4 note
    assert!(freq.is_ok());
}

#[test]
fn test_frequency_new_negative_error() {
    let freq = Frequency::new(-1.0);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_unchecked() {
    let freq = Frequency::new_unchecked(60.0);
    assert!((freq.value() - 60.0).abs() < 1e-10);
}

#[test]
fn test_frequency_into_f64() {
    let freq = Frequency::new(50.0).unwrap();
    let val: f64 = freq.into();
    assert!((val - 50.0).abs() < 1e-10);
}

// =============================================================================
// Acceleration Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_acceleration_new_positive() {
    let acc = Acceleration::new(9.81);
    assert!(acc.is_ok());
}

#[test]
fn test_acceleration_new_negative() {
    // Negative acceleration is valid (deceleration)
    let acc = Acceleration::new(-5.0);
    assert!(acc.is_ok());
    assert!((acc.unwrap().value() - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_acceleration_new_unchecked() {
    let acc = Acceleration::new_unchecked(-9.81);
    assert!((acc.value() - (-9.81)).abs() < 1e-10);
}

#[test]
fn test_acceleration_into_f64() {
    let acc = Acceleration::new(9.8).unwrap();
    let val: f64 = acc.into();
    assert!((val - 9.8).abs() < 1e-10);
}

// =============================================================================
// Force Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_force_new_positive() {
    let force = Force::new(100.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_negative() {
    let force = Force::new(-50.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_unchecked() {
    let force = Force::new_unchecked(-100.0);
    assert!((force.value() - (-100.0)).abs() < 1e-10);
}

#[test]
fn test_force_into_f64() {
    let force = Force::new(10.0).unwrap();
    let val: f64 = force.into();
    assert!((val - 10.0).abs() < 1e-10);
}

// =============================================================================
// Torque Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_torque_new_positive() {
    let torque = Torque::new(25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_negative() {
    // Negative torque = clockwise rotation
    let torque = Torque::new(-25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_unchecked() {
    let torque = Torque::new_unchecked(-25.0);
    assert!((torque.value() - (-25.0)).abs() < 1e-10);
}

#[test]
fn test_torque_into_f64() {
    let torque = Torque::new(50.0).unwrap();
    let val: f64 = torque.into();
    assert!((val - 50.0).abs() < 1e-10);
}
