// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let e = MinkowskiSpacetime::new(99, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(e.id(), 99);
}

#[test]
fn test_coordinate_trait() {
    let e = MinkowskiSpacetime::new(1, 3.0, 4.0, 5.0, 6.0, TimeScale::Second);

    assert_eq!(e.dimension(), 4);
    assert_eq!(*e.coordinate(0), 3.0);
    assert_eq!(*e.coordinate(1), 4.0);
    assert_eq!(*e.coordinate(2), 5.0);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_coordinate_out_of_bounds() {
    let e = MinkowskiSpacetime::new(1, 3.0, 4.0, 5.0, 6.0, TimeScale::Second);
    let _ = e.coordinate(4);
}

#[test]
fn test_display_trait() {
    let e = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Millisecond);
    let text = format!("{e}");
    assert!(text.contains("MinkowskiSpacetime"));
    assert!(text.contains("x=1.0"));
    assert!(text.contains("t=4.0"));
}

#[test]
fn test_temporal_trait() {
    let e = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 42.0, TimeScale::Second);

    assert_eq!(e.time_scale(), TimeScale::Second);
    assert_eq!(e.time_unit(), 42.0);
}

#[test]
fn test_space_temporal_trait() {
    let e = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 9999.0, TimeScale::Second);

    assert_eq!(e.t(), &9999.0);
    assert_eq!(*e.coordinate(1), 2.0);
}

#[test]
fn test_interval_squared_time_like() {
    let e1 = MinkowskiSpacetime::new(1, 1.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let e2 = MinkowskiSpacetime::new(2, 3.0, 3.0, 4.0, 1.0, TimeScale::Second);

    let s2 = e1.interval_squared(&e2);
    assert!(s2 < 0.0, "Expected time-like interval, got s² = {s2}");
}

#[test]
fn test_interval_squared_space_like() {
    let e1 = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let e2 = MinkowskiSpacetime::new(2, 100.0, 0.0, 0.0, 0.0000001, TimeScale::Second);

    let s2 = e1.interval_squared(&e2);
    assert!(s2 > 0.0, "Expected space-like interval, got s² = {s2}");
}

#[test]
fn test_interval_squared_light_like() {
    let c = 299_792_458.0;
    let dt = 1.0;
    let dx = c * dt;

    let e1 = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let e2 = MinkowskiSpacetime::new(2, dx, 0.0, 0.0, dt, TimeScale::Second);

    let s2 = e1.interval_squared(&e2);
    let epsilon = 1e-6;
    assert!(
        s2.abs() < epsilon,
        "Expected light-like interval, got s² = {s2}"
    );
}
