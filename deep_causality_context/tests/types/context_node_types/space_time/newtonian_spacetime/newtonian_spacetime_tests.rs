/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let s = NewtonianSpacetime::new(42, 1.0, 2.0, 3.0, 1_000_000.0, TimeScale::Second);
    assert_eq!(s.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);

    // Time first: 0 => t, 1 => x, 2 => y, 3 => z.
    assert_eq!(s.dimension(), 4);
    assert_eq!(*s.coordinate(0).unwrap(), 0.0);
    assert_eq!(*s.coordinate(1).unwrap(), 1.0);
    assert_eq!(*s.coordinate(2).unwrap(), 2.0);
    assert_eq!(*s.coordinate(3).unwrap(), 3.0);
}

#[test]
fn test_coordinate_trait_out_of_bounds() {
    let s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);
    let res = s.coordinate(4);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 1_000_000.0, TimeScale::Second);
    let formatted = format!("{s}");
    dbg!(&formatted);
    assert!(formatted.contains("NewtonianSpacetime"));
    assert!(formatted.contains("x=1.0"));
    assert!(formatted.contains("t=1000000"));
}

#[test]
fn test_metric_trait() {
    let a = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let b = NewtonianSpacetime::new(2, 3.0, 4.0, 0.0, 0.0, TimeScale::Second);

    let dist = a.distance(&b);
    assert_eq!(dist, 5.0);
}

#[test]
fn test_distance_in_absolute_space_ignores_the_time_difference() {
    // A rest frame makes places persist: the same place a minute later is at distance zero, and
    // a 3-4-12 displacement at any time difference is 13 m.
    let a = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let later = NewtonianSpacetime::new(2, 0.0, 0.0, 0.0, 60.0, TimeScale::Second);
    let away = NewtonianSpacetime::new(3, 3.0, 4.0, 12.0, 5.0, TimeScale::Hour);
    assert_eq!(a.distance(&later), 0.0);
    assert_eq!(a.distance(&away), 13.0);
}

#[test]
fn test_temporal_trait() {
    let s = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 123456.0, TimeScale::Second);

    assert_eq!(s.time_scale(), TimeScale::Second);
    assert_eq!(s.time_unit(), 123456.0);
}

#[test]
fn test_space_temporal_trait() {
    let s = NewtonianSpacetime::new(1, 9.0, 8.0, 7.0, 654321.0, TimeScale::Second);

    assert_eq!(s.t(), &654321.0);
    assert_eq!(*s.coordinate(0).unwrap(), 654321.0);
    assert_eq!(*s.coordinate(1).unwrap(), 9.0);
}

#[test]
fn test_distance_between_two_events_away_from_the_origin() {
    // Row C: neither event at the origin. (1, 2, 3) to (4, 6, 15) is the 3-4-12 triangle, 13 m,
    // at any time difference.
    let a: NewtonianSpacetime<f64> =
        NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 7.0, TimeScale::Second);
    let b = NewtonianSpacetime::new(2, 4.0, 6.0, 15.0, 9.0, TimeScale::Hour);
    assert_eq!(a.distance(&b), 13.0);
    assert_eq!(b.distance(&a), 13.0);
}
