/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_identifiable_trait() {
    let s = TangentSpacetime::new(42, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(s.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);

    assert_eq!(s.dimension(), 4);
    assert_eq!(*s.coordinate(0).unwrap(), 1.0);
    assert_eq!(*s.coordinate(1).unwrap(), 2.0);
    assert_eq!(*s.coordinate(2).unwrap(), 3.0);
    assert_eq!(*s.coordinate(3).unwrap(), 4.0);
}

#[test]
fn test_coordinate_index_out_of_bounds() {
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    let res = s.coordinate(5);
    assert!(res.is_err());
}

#[test]
fn test_temporal_trait() {
    let s = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 12345.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(s.time_unit(), 12345.0);
    assert_eq!(s.time_scale(), TimeScale::Second);
}

#[test]
fn test_space_temporal_trait() {
    let s = TangentSpacetime::new(1, 4.0, 5.0, 6.0, 42.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(s.t(), &42.0);
    assert_eq!(*s.coordinate(0).unwrap(), 4.0);

    let time = s.time();
    assert_eq!(time, 42.0);
}

#[test]
fn test_display_trait() {
    let s = TangentSpacetime::new(1, 1.0, 0.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0);
    let out = format!("{s}");
    assert!(out.contains("TangentSpacetime"));
    assert!(out.contains("x=1.0"));
    assert!(out.contains("t=2.0"));
}

#[test]
fn test_metric_tensor_trait() {
    let mut s = TangentSpacetime::new(1, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let original = s.metric_tensor();
    assert_eq!(original[0][0], -8.987551787368176e16);

    let warped = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.05, 0.0, 0.0],
        [0.0, 0.0, 0.95, 0.0],
        [0.0, 0.0, 0.0, 0.90],
    ];

    s.update_metric_tensor(warped);
    let updated = s.metric_tensor();
    assert_eq!(updated[1][1], 1.05);
    assert_eq!(updated[3][3], 0.90);
}

#[test]
fn test_spacetime_interval_trait() {
    let s1 = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let s2 = TangentSpacetime::new(2, 3.0, 4.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0);

    // Minkowski metric used by default
    let s2_interval = s1.interval_squared(&s2);

    // Should be time-like
    assert!(s2_interval < 0.0);
}

#[test]
fn test_spacetime_interval_with_custom_metric() {
    let mut s1 = TangentSpacetime::new(1, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);

    // Apply warped metric
    let warped = [
        [-8.98755179e16, 0.0, 0.0, 0.0],
        [0.0, 1.05, 0.0, 0.0],
        [0.0, 0.0, 0.95, 0.0],
        [0.0, 0.0, 0.0, 0.90],
    ];
    s1.update_metric_tensor(warped);

    let s2 = TangentSpacetime::new(2, 2.0, 3.0, 4.0, 0.0, 1.0, 0.0, 0.0, 0.0);

    let interval = s1.interval_squared(&s2);
    println!("Curved spacetime interval² = {interval}");
    assert!(interval > 0.0); // space-like under warped metric
}

#[test]
fn test_get_x() {
    let t = TangentSpacetime::new(1, 42.0, 0.0, 0.0, 0.0, 1.0, 0.1, 0.1, 0.1);
    assert_eq!(t.x(), 42.0);
}

#[test]
fn test_get_y() {
    let t = TangentSpacetime::new(1, 0.0, 99.0, 0.0, 0.0, 1.0, 0.1, 0.1, 0.1);
    assert_eq!(t.y(), 99.0);
}

#[test]
fn test_get_z() {
    let t = TangentSpacetime::new(1, 0.0, 0.0, 123.0, 0.0, 1.0, 0.1, 0.1, 0.1);
    assert_eq!(t.z(), 123.0);
}

#[test]
fn test_position() {
    let t = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(t.position(), [1.0, 2.0, 3.0]);
}

#[test]
fn test_velocity() {
    let t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 10.0, 1.0, 2.0, 3.0);
    assert_eq!(t.velocity(), [10.0, 1.0, 2.0, 3.0]);
}

#[test]
fn test_time_velocity() {
    let t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 123.456, 0.0, 0.0, 0.0);
    assert_eq!(t.time_velocity(), 123.456);
}

#[test]
fn test_spatial_velocity() {
    let t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 4.0, 12.0);
    let expected = (3.0_f64.powi(2) + 4.0_f64.powi(2) + 12.0_f64.powi(2)).sqrt();
    assert_eq!(t.spatial_velocity(), expected);
}

#[test]
fn test_velocity_vector() {
    let t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0, 8.0, 9.0);
    assert_eq!(t.velocity_vector(), [7.0, 8.0, 9.0]);
}

#[test]
fn test_euclidean_distance() {
    let t1 = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let t2 = TangentSpacetime::new(2, 4.0, 6.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let expected = ((4.0f64 - 1.0f64).powi(2) + (6.0f64 - 2.0f64).powi(2)).sqrt();
    assert_eq!(t1.euclidean_distance(&t2), expected);
}
