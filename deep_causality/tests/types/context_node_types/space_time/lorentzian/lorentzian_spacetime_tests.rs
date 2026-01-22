/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_identifiable_trait() {
    let s = LorentzianSpacetime::new(7, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(s.id(), 7);
}

#[test]
fn test_coordinate_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);

    assert_eq!(s.dimension(), 4);
    assert_eq!(*s.coordinate(0).unwrap(), 1.0);
    assert_eq!(*s.coordinate(1).unwrap(), 2.0);
    assert_eq!(*s.coordinate(2).unwrap(), 3.0);
}

#[test]
fn test_coordinate_trait_out_of_bounds() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);
    let res = s.coordinate(4);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, TimeScale::Millisecond);
    let formatted = format!("{s}");
    dbg!(&formatted);
    assert!(formatted.contains("LorentzianSpacetime"));
    assert!(formatted.contains("x=1.0"));
    assert!(formatted.contains("t=42.0"));
}

#[test]
fn test_temporal_trait() {
    let s = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 123456.0, TimeScale::Second);

    assert_eq!(s.time_scale(), TimeScale::Second);
    assert_eq!(s.time_unit(), 123456.0);
}

#[test]
fn test_space_temporal_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 999999.0, TimeScale::Second);

    // dbg!(*s.t());
    assert_eq!(s.t(), &999999.0);
    // dbg!(*s.coordinate(0));
    assert_eq!(*s.coordinate(0).unwrap(), 1.0);
}

#[test]
fn test_lorentzian_time_and_position() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, TimeScale::Millisecond);

    // time() returns seconds: 42 milliseconds = 0.042 seconds
    assert_eq!(s.time(), 0.042);
    assert_eq!(s.position(), [1.0, 2.0, 3.0]);
}

#[test]
fn test_interval_squared_timelike_is_negative() {
    // Two events separated by time but not space.
    let a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 10.0, TimeScale::Second);
    let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    let result = a.interval_squared(&b);
    // s² = -c²·Δt² + Δx² = -c²·(10)² + 0 < 0
    assert!(result < 0.0, "Expected time-like interval to be negative");
}

#[test]
fn test_interval_squared_spacelike_is_positive() {
    // Two events separated by space but not time.
    let a = LorentzianSpacetime::new(1, 3.0e3, 0.0, 0.0, 0.0, TimeScale::Second);
    let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    let result = a.interval_squared(&b);
    // s² = -c²·Δt² + Δx² = 0 + (3e3)² > 0
    assert!(result > 0.0, "Expected space-like interval to be positive");
}

#[test]
fn test_interval_squared_null_like() {
    let c = 299_792_458.0;
    let a = LorentzianSpacetime::new(1, c, 0.0, 0.0, 1.0, TimeScale::Second);
    let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    let result = a.interval_squared(&b);
    assert!(result.abs() < 1e-6); // null-like
}
