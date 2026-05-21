/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
            assert!(msg.contains("negative") || msg.contains("Mass"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_mass_new_nan_error() {
    let mass = Mass::<f64>::new(f64::NAN);
    assert!(mass.is_err());
}

#[test]
fn test_mass_new_infinity_error() {
    let mass = Mass::<f64>::new(f64::INFINITY);
    assert!(mass.is_err());
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

// =============================================================================
// Speed Tests
// =============================================================================

#[test]
fn test_speed_new_valid() {
    let speed = Speed::<f64>::new(100.0);
    assert!(speed.is_ok());
    assert!((speed.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_speed_new_zero() {
    let speed = Speed::<f64>::new(0.0);
    assert!(speed.is_ok());
}

#[test]
fn test_speed_new_negative_error() {
    let speed = Speed::<f64>::new(-50.0);
    assert!(speed.is_err());
}

#[test]
fn test_speed_new_nan_error() {
    let speed = Speed::<f64>::new(f64::NAN);
    assert!(speed.is_err());
}

#[test]
fn test_speed_new_infinity_error() {
    let speed = Speed::<f64>::new(f64::INFINITY);
    assert!(speed.is_err());
}

#[test]
fn test_speed_into_f64() {
    let speed = Speed::<f64>::new(299792458.0).unwrap();
    let val: f64 = speed.into();
    assert!((val - 299792458.0).abs() < 1.0);
}

#[test]
fn test_speed_new_unchecked() {
    let speed = Speed::<f64>::new_unchecked(123.45);
    assert!((speed.value() - 123.45).abs() < 1e-10);
}

// =============================================================================
// Length Tests
// =============================================================================

#[test]
fn test_length_new_valid() {
    let length = Length::<f64>::new(1.5e11);
    assert!(length.is_ok());
}

#[test]
fn test_length_new_negative_error() {
    let length = Length::<f64>::new(-1.0);
    assert!(length.is_err());
}

#[test]
fn test_length_new_nan_error() {
    let length = Length::<f64>::new(f64::NAN);
    assert!(length.is_err());
}

#[test]
fn test_length_new_infinity_error() {
    let length = Length::<f64>::new(f64::INFINITY);
    assert!(length.is_err());
}

#[test]
fn test_length_default() {
    let length = Length::<f64>::default();
    assert!((length.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_length_new_unchecked() {
    let length = Length::<f64>::new_unchecked(99.9);
    assert!((length.value() - 99.9).abs() < 1e-10);
}

#[test]
fn test_length_into_f64() {
    let length = Length::<f64>::new(10.0).unwrap();
    let val: f64 = length.into();
    assert!((val - 10.0).abs() < 1e-10);
}

// =============================================================================
// Area Tests
// =============================================================================

#[test]
fn test_area_new_valid() {
    let area = Area::<f64>::new(100.0);
    assert!(area.is_ok());
    assert!((area.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_area_new_negative_error() {
    let area = Area::<f64>::new(-10.0);
    assert!(area.is_err());
}

#[test]
fn test_area_new_nan_error() {
    let area = Area::<f64>::new(f64::NAN);
    assert!(area.is_err());
}

#[test]
fn test_area_new_infinity_error() {
    let area = Area::<f64>::new(f64::INFINITY);
    assert!(area.is_err());
}

#[test]
fn test_area_new_unchecked() {
    let area = Area::<f64>::new_unchecked(50.5);
    assert!((area.value() - 50.5).abs() < 1e-10);
}

#[test]
fn test_area_into_f64() {
    let area = Area::<f64>::new(25.0).unwrap();
    let val: f64 = area.into();
    assert!((val - 25.0).abs() < 1e-10);
}

// =============================================================================
// Volume Tests
// =============================================================================

#[test]
fn test_volume_new_valid() {
    let volume = Volume::<f64>::new(1000.0);
    assert!(volume.is_ok());
}

#[test]
fn test_volume_new_negative_error() {
    let volume = Volume::<f64>::new(-1.0);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_nan_error() {
    let volume = Volume::<f64>::new(f64::NAN);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_infinity_error() {
    let volume = Volume::<f64>::new(f64::INFINITY);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_unchecked() {
    let volume = Volume::<f64>::new_unchecked(123.0);
    assert!((volume.value() - 123.0).abs() < 1e-10);
}

#[test]
fn test_volume_into_f64() {
    let volume = Volume::<f64>::new(100.0).unwrap();
    let val: f64 = volume.into();
    assert!((val - 100.0).abs() < 1e-10);
}

// =============================================================================
// MomentOfInertia Tests
// =============================================================================

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

// =============================================================================
// Frequency Tests
// =============================================================================

#[test]
fn test_frequency_new_valid() {
    let freq = Frequency::<f64>::new(440.0); // A4 note
    assert!(freq.is_ok());
}

#[test]
fn test_frequency_new_negative_error() {
    let freq = Frequency::<f64>::new(-1.0);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_nan_error() {
    let freq = Frequency::<f64>::new(f64::NAN);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_infinity_error() {
    let freq = Frequency::<f64>::new(f64::INFINITY);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_unchecked() {
    let freq = Frequency::<f64>::new_unchecked(60.0);
    assert!((freq.value() - 60.0).abs() < 1e-10);
}

#[test]
fn test_frequency_into_f64() {
    let freq = Frequency::<f64>::new(50.0).unwrap();
    let val: f64 = freq.into();
    assert!((val - 50.0).abs() < 1e-10);
}

// =============================================================================
// Acceleration Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_acceleration_new_positive() {
    let acc = Acceleration::<f64>::new(9.81);
    assert!(acc.is_ok());
}

#[test]
fn test_acceleration_new_negative() {
    // Negative acceleration is valid (deceleration)
    let acc = Acceleration::<f64>::new(-5.0);
    assert!(acc.is_ok());
    assert!((acc.unwrap().value() - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_acceleration_new_nan_error() {
    let acc = Acceleration::<f64>::new(f64::NAN);
    assert!(acc.is_err());
}

#[test]
fn test_acceleration_new_infinity_error() {
    let acc = Acceleration::<f64>::new(f64::INFINITY);
    assert!(acc.is_err());
}

#[test]
fn test_acceleration_new_unchecked() {
    let acc = Acceleration::<f64>::new_unchecked(-9.81);
    assert!((acc.value() - (-9.81)).abs() < 1e-10);
}

#[test]
fn test_acceleration_into_f64() {
    let acc = Acceleration::<f64>::new(9.8).unwrap();
    let val: f64 = acc.into();
    assert!((val - 9.8).abs() < 1e-10);
}

// =============================================================================
// Force Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_force_new_positive() {
    let force = Force::<f64>::new(100.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_negative() {
    let force = Force::<f64>::new(-50.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_nan_error() {
    let force = Force::<f64>::new(f64::NAN);
    assert!(force.is_err());
}

#[test]
fn test_force_new_infinity_error() {
    let force = Force::<f64>::new(f64::INFINITY);
    assert!(force.is_err());
}

#[test]
fn test_force_new_unchecked() {
    let force = Force::<f64>::new_unchecked(-100.0);
    assert!((force.value() - (-100.0)).abs() < 1e-10);
}

#[test]
fn test_force_into_f64() {
    let force = Force::<f64>::new(10.0).unwrap();
    let val: f64 = force.into();
    assert!((val - 10.0).abs() < 1e-10);
}

// =============================================================================
// Torque Tests (allows negative for direction)
// =============================================================================

#[test]
fn test_torque_new_positive() {
    let torque = Torque::<f64>::new(25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_negative() {
    // Negative torque = clockwise rotation
    let torque = Torque::<f64>::new(-25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_nan_error() {
    let torque = Torque::<f64>::new(f64::NAN);
    assert!(torque.is_err());
}

#[test]
fn test_torque_new_infinity_error() {
    let torque = Torque::<f64>::new(f64::INFINITY);
    assert!(torque.is_err());
}

#[test]
fn test_torque_new_unchecked() {
    let torque = Torque::<f64>::new_unchecked(-25.0);
    assert!((torque.value() - (-25.0)).abs() < 1e-10);
}

#[test]
fn test_torque_into_f64() {
    let torque = Torque::<f64>::new(50.0).unwrap();
    let val: f64 = torque.into();
    assert!((val - 50.0).abs() < 1e-10);
}

// =============================================================================
// Default impls — covering the remaining scalars
// =============================================================================

#[test]
fn test_speed_default() {
    let s: Speed<f64> = Speed::default();
    assert_eq!(s.value(), 0.0);
}

#[test]
fn test_acceleration_default() {
    let a: Acceleration<f64> = Acceleration::default();
    assert_eq!(a.value(), 0.0);
}

#[test]
fn test_force_default() {
    let f: Force<f64> = Force::default();
    assert_eq!(f.value(), 0.0);
}

#[test]
fn test_torque_default() {
    let t: Torque<f64> = Torque::default();
    assert_eq!(t.value(), 0.0);
}

#[test]
fn test_area_default() {
    let a: Area<f64> = Area::default();
    assert_eq!(a.value(), 0.0);
}

#[test]
fn test_volume_default() {
    let v: Volume<f64> = Volume::default();
    assert_eq!(v.value(), 0.0);
}

#[test]
fn test_moment_of_inertia_default() {
    let m: MomentOfInertia<f64> = MomentOfInertia::default();
    assert_eq!(m.value(), 0.0);
}

#[test]
fn test_frequency_default() {
    let f: Frequency<f64> = Frequency::default();
    assert_eq!(f.value(), 0.0);
}

// =============================================================================
// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd
// =============================================================================

#[test]
fn test_dynamics_scalars_traits() {
    let m1 = Mass::<f64>::new(1.0).unwrap();
    let m2 = m1;
    let m3 = m1.clone();
    assert_eq!(m1, m2);
    assert_eq!(m1, m3);
    assert!(m1 < Mass::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", m1);

    let s = Speed::<f64>::new(3.0).unwrap();
    assert!(s < Speed::<f64>::new(4.0).unwrap());
    let _ = format!("{:?}", s);

    let a = Acceleration::<f64>::new(-1.0).unwrap();
    let _ = format!("{:?}", a);
    assert_eq!(a.clone(), a);

    let f = Force::<f64>::new(-5.0).unwrap();
    let _ = format!("{:?}", f);
    assert_eq!(f, f.clone());

    let t = Torque::<f64>::new(2.0).unwrap();
    let _ = format!("{:?}", t);
    assert_eq!(t, t.clone());

    let l = Length::<f64>::new(2.0).unwrap();
    assert!(l < Length::<f64>::new(3.0).unwrap());
    let _ = format!("{:?}", l);

    let ar = Area::<f64>::new(4.0).unwrap();
    assert!(ar < Area::<f64>::new(5.0).unwrap());
    let _ = format!("{:?}", ar);

    let v = Volume::<f64>::new(1.0).unwrap();
    assert!(v < Volume::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", v);

    let i = MomentOfInertia::<f64>::new(0.5).unwrap();
    assert!(i < MomentOfInertia::<f64>::new(1.0).unwrap());
    let _ = format!("{:?}", i);

    let fr = Frequency::<f64>::new(60.0).unwrap();
    assert!(fr < Frequency::<f64>::new(120.0).unwrap());
    let _ = format!("{:?}", fr);
}
