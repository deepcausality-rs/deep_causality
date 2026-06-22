/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Temperature;

#[test]
fn test_temperature_new_valid() {
    let t = Temperature::<f64>::new(300.0);
    assert!(t.is_ok());
    assert!((t.unwrap().value() - 300.0).abs() < 1e-10);
}

#[test]
fn test_temperature_new_zero_kelvin() {
    let t = Temperature::<f64>::new(0.0);
    assert!(t.is_ok());
}

#[test]
fn test_temperature_new_negative_error() {
    let t = Temperature::<f64>::new(-1.0);
    assert!(t.is_err(), "Negative Kelvin should error");
}

#[test]
fn test_temperature_from_celsius() {
    // 0°C = 273.15K
    let t = Temperature::<f64>::from_celsius(0.0).unwrap();
    assert!((t.value() - 273.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_celsius_boiling() {
    // 100°C = 373.15K
    let t = Temperature::<f64>::from_celsius(100.0).unwrap();
    assert!((t.value() - 373.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_fahrenheit() {
    // 32°F = 0°C = 273.15K
    let t = Temperature::<f64>::from_fahrenheit(32.0).unwrap();
    assert!((t.value() - 273.15).abs() < 1e-10);
}

#[test]
fn test_temperature_from_fahrenheit_boiling() {
    // 212°F = 100°C = 373.15K
    let t = Temperature::<f64>::from_fahrenheit(212.0).unwrap();
    assert!((t.value() - 373.15).abs() < 1e-10);
}

#[test]
fn test_temperature_as_celsius() {
    let t = Temperature::<f64>::new(273.15).unwrap();
    assert!((t.as_celsius() - 0.0).abs() < 1e-10);
}

#[test]
fn test_temperature_as_fahrenheit() {
    let t = Temperature::<f64>::new(273.15).unwrap();
    assert!((t.as_fahrenheit() - 32.0).abs() < 1e-10);
}

#[test]
fn test_temperature_into_f64() {
    let t = Temperature::<f64>::new_unchecked(300.0);
    let val: f64 = t.into();
    assert!((val - 300.0).abs() < 1e-10);
}

#[test]
fn test_temperature_default() {
    // si_primitives/mod.rs:248-250
    let t = Temperature::<f64>::default();
    assert!((t.value() - 0.0).abs() < 1e-10);
}

// NOTE on si_primitives/mod.rs:274-275 — the `ok_or_else` closure body for
// `R::from_f64(ZERO_CELSIUS_IN_KELVIN)` in `Temperature::from_celsius` (also
// reached transitively by `from_fahrenheit`). `from_f64` is infallible for f64,
// so the conversion never returns `None` and this defensive error closure can
// never run for the f64 monomorphisation. The success path of `from_celsius`
// is covered by the conversion tests above.
