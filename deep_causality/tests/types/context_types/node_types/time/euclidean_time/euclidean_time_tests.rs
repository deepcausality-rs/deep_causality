/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::prelude::*;

#[test]
fn test_euclidean_time_construction() {
    let t = EuclideanTime::new(1, TimeScale::Second, 42.0);
    assert_eq!(t.id(), 1);
    assert_eq!(t.time_scale(), TimeScale::Second);
    assert_eq!(t.time_unit(), 42.0);
}

#[test]
fn test_euclidean_time_identifiable_trait() {
    let t = EuclideanTime::new(99, TimeScale::Millisecond, 1234.5);
    assert_eq!(t.id(), 99);
}

#[test]
fn test_euclidean_time_temporal_trait() {
    let t = EuclideanTime::new(5, TimeScale::Microseconds, 10.5);
    assert_eq!(t.time_scale(), TimeScale::Microseconds);
    assert_eq!(t.time_unit(), 10.5);
}

#[test]
fn test_euclidean_time_scalar_projector_trait() {
    let t = EuclideanTime::new(7, TimeScale::Nanoseconds, 0.001);
    let scalar: f64 = t.project();
    assert_eq!(scalar, 0.001);
}

#[test]
fn test_euclidean_time_display_trait() {
    let t = EuclideanTime::new(1, TimeScale::Second, 3.00);
    let output = format!("{}", t);
    assert!(
        output.contains("EuclideanTime"),
        "Expected display output to include struct name"
    );
    assert!(
        output.contains("3.00"),
        "Expected display output to include time value"
    );
    assert!(
        output.contains("Second"),
        "Expected display output to include time scale"
    );
}
