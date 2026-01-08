/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Energy;

#[test]
fn test_energy_new_valid() {
    let e = Energy::new(100.0);
    assert!(e.is_ok());
    assert!((e.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_energy_new_negative() {
    // Energy can be negative (potential wells)
    let e = Energy::new(-50.0);
    assert!(e.is_ok());
}

#[test]
fn test_energy_from_electron_volts() {
    let e = Energy::from_electron_volts(1.0).unwrap();
    // 1 eV = 1.602e-19 J
    assert!((e.value() - 1.602_176_634e-19).abs() < 1e-28);
}

#[test]
fn test_energy_from_calories() {
    let e = Energy::from_calories(1.0).unwrap();
    // 1 cal = 4.184 J
    assert!((e.value() - 4.184).abs() < 1e-10);
}

#[test]
fn test_energy_from_kilowatt_hours() {
    let e = Energy::from_kilowatt_hours(1.0).unwrap();
    // 1 kWh = 3.6e6 J
    assert!((e.value() - 3.6e6).abs() < 1e-3);
}

#[test]
fn test_energy_as_electron_volts_roundtrip() {
    let e = Energy::from_electron_volts(1000.0).unwrap();
    let ev = e.as_electron_volts();
    assert!((ev - 1000.0).abs() < 1e-6);
}

#[test]
fn test_energy_as_calories_roundtrip() {
    let e = Energy::from_calories(100.0).unwrap();
    let cal = e.as_calories();
    assert!((cal - 100.0).abs() < 1e-10);
}

#[test]
fn test_energy_as_kilowatt_hours_roundtrip() {
    let e = Energy::from_kilowatt_hours(2.0).unwrap();
    let kwh = e.as_kilowatt_hours();
    assert!((kwh - 2.0).abs() < 1e-10);
}

#[test]
fn test_energy_into_f64() {
    let e = Energy::new_unchecked(42.0);
    let val: f64 = e.into();
    assert!((val - 42.0).abs() < 1e-10);
}
