// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let s = EuclideanSpacetime::new(42, [1.0, 2.0, 3.0], 1_000_000.0, TimeScale::Second);
    assert_eq!(s.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let s = EuclideanSpacetime::new(1, [1.0, 2.0, 3.0], 0.0, TimeScale::Second);

    assert_eq!(s.dimension(), 3);
    assert_eq!(*s.coordinate(0), 1.0);
    assert_eq!(*s.coordinate(1), 2.0);
    assert_eq!(*s.coordinate(2), 3.0);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_coordinate_trait_out_of_bounds() {
    let s = EuclideanSpacetime::new(1, [1.0, 2.0, 3.0], 0.0, TimeScale::Second);
    let _ = s.coordinate(3); // Should panic
}

#[test]
fn test_display_trait() {
    let s = EuclideanSpacetime::new(1, [1.0, 2.0, 3.0], 1_000_000.0, TimeScale::Second);
    let formatted = format!("{s}");
    dbg!(&formatted);
    assert!(formatted.contains("EuclideanSpacetime"));
    assert!(formatted.contains("x=1.0"));
    assert!(formatted.contains("t=1000000"));
}

#[test]
fn test_metric_trait() {
    let a = EuclideanSpacetime::new(1, [0.0, 0.0, 0.0], 0.0, TimeScale::Second);
    let b = EuclideanSpacetime::new(2, [3.0, 4.0, 0.0], 0.0, TimeScale::Second);

    let dist = a.distance(&b);
    assert_eq!(dist, 5.0);
}

#[test]
fn test_temporal_trait() {
    let s = EuclideanSpacetime::new(1, [0.0, 0.0, 0.0], 123456.0, TimeScale::Second);

    assert_eq!(s.time_scale(), TimeScale::Second);
    assert_eq!(s.time_unit(), 123456.0);
}

#[test]
fn test_space_temporal_trait() {
    let s = EuclideanSpacetime::new(1, [9.0, 8.0, 7.0], 654321.0, TimeScale::Second);

    assert_eq!(s.t(), &654321.0);
    assert_eq!(*s.coordinate(0), 9.0);
}
