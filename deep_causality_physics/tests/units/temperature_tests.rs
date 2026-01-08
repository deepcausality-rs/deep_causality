/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Temperature;

#[test]
fn test_temperature_new_valid() {
    let t = Temperature::new(300.0);
    assert!(t.is_ok());
    assert!((t.unwrap().value() - 300.0).abs() < 1e-10);
}

#[test]
fn test_temperature_new_zero_kelvin() {
    let t = Temperature::new(0.0);
    assert!(t.is_ok());
}

#[test]
fn test_temperature_new_negative_error() {
    let t = Temperature::new(-1.0);
    assert!(t.is_err(), "Negative Kelvin should error");
}

#[test]
fn test_temperature_from_celsius() {
    // 0°C = 273.15K
    let t = Temperature::from_celsius(0.0).unwrap();
    assert!((t.value() - 273.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_celsius_boiling() {
    // 100°C = 373.15K
    let t = Temperature::from_celsius(100.0).unwrap();
    assert!((t.value() - 373.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_fahrenheit() {
    // 32°F = 0°C = 273.15K
    let t = Temperature::from_fahrenheit(32.0).unwrap();
    assert!((t.value() - 273.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_fahrenheit_boiling() {
    // 212°F = 100°C = 373.15K
    let t = Temperature::from_fahrenheit(212.0).unwrap();
    assert!((t.value() - 373.15).abs() < 1e-10);
}

#[test]
fn test_temperature_as_celsius() {
    let t = Temperature::new(273.15).unwrap();
    assert!((t.as_celsius() - 0.0).abs() < 1e-10);
}

#[test]
fn test_temperature_as_fahrenheit() {
    let t = Temperature::new(273.15).unwrap();
    assert!((t.as_fahrenheit() - 32.0).abs() < 1e-10);
}

#[test]
fn test_temperature_into_f64() {
    let t = Temperature::new_unchecked(300.0);
    let val: f64 = t.into();
    assert!((val - 300.0).abs() < 1e-10);
}
