/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Time;

// =============================================================================
// Time Constructor Tests
// =============================================================================

#[test]
fn test_time_new_valid() {
    let t = Time::new(60.0);
    assert!(t.is_ok());
    assert!((t.unwrap().value() - 60.0).abs() < 1e-10);
}

#[test]
fn test_time_new_zero() {
    let t = Time::new(0.0);
    assert!(t.is_ok());
    assert!((t.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_time_new_negative_error() {
    let t = Time::new(-1.0);
    assert!(t.is_err(), "Negative time should error");
}

#[test]
fn test_time_new_unchecked() {
    let t = Time::new_unchecked(100.0);
    assert!((t.value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_time_default() {
    let t = Time::default();
    assert!((t.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// Time Conversion Tests
// =============================================================================

#[test]
fn test_time_from_minutes() {
    // 1 minute = 60 seconds
    let t = Time::from_minutes(1.0).unwrap();
    assert!((t.value() - 60.0).abs() < 1e-10);
}

#[test]
fn test_time_from_hours() {
    // 1 hour = 3600 seconds
    let t = Time::from_hours(1.0).unwrap();
    assert!((t.value() - 3600.0).abs() < 1e-10);
}

#[test]
fn test_time_from_days() {
    // 1 day = 86400 seconds
    let t = Time::from_days(1.0).unwrap();
    assert!((t.value() - 86400.0).abs() < 1e-10);
}

#[test]
fn test_time_from_years() {
    // 1 Julian year = 365.25 days = 31,557,600 seconds
    let t = Time::from_years(1.0).unwrap();
    assert!((t.value() - 31_557_600.0).abs() < 1e-3);
}

#[test]
fn test_time_as_minutes_roundtrip() {
    let t = Time::from_minutes(5.0).unwrap();
    assert!((t.as_minutes() - 5.0).abs() < 1e-10);
}

#[test]
fn test_time_as_hours_roundtrip() {
    let t = Time::from_hours(2.0).unwrap();
    assert!((t.as_hours() - 2.0).abs() < 1e-10);
}

#[test]
fn test_time_as_days_roundtrip() {
    let t = Time::from_days(7.0).unwrap();
    assert!((t.as_days() - 7.0).abs() < 1e-10);
}

#[test]
fn test_time_as_years_roundtrip() {
    let t = Time::from_years(1.5).unwrap();
    assert!((t.as_years() - 1.5).abs() < 1e-10);
}

#[test]
fn test_time_into_f64() {
    let t = Time::new(123.0).unwrap();
    let val: f64 = t.into();
    assert!((val - 123.0).abs() < 1e-10);
}
