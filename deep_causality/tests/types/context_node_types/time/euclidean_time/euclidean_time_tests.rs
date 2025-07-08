/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_construction() {
    let t = EuclideanTime::new(1, TimeScale::Second, 42.0);
    assert_eq!(t.id(), 1);
    assert_eq!(t.time_scale(), TimeScale::Second);
    assert_eq!(t.time_unit(), 42.0);
}

#[test]
fn test_identifiable_trait() {
    let t = EuclideanTime::new(99, TimeScale::Millisecond, 1234.5);
    assert_eq!(t.id(), 99);
}

#[test]
fn test_temporal_trait() {
    let t = EuclideanTime::new(5, TimeScale::Microseconds, 10.5);
    assert_eq!(t.time_scale(), TimeScale::Microseconds);
    assert_eq!(t.time_unit(), 10.5);
}

#[test]
fn test_scalar_projector_trait() {
    let t = EuclideanTime::new(7, TimeScale::Nanoseconds, 0.001);
    let scalar: f64 = t.project();
    assert_eq!(scalar, 0.001);
}

#[test]
fn test_from_euclidean_time_to_time_kind() {
    let time = EuclideanTime::new(1, TimeScale::Second, 3.00);
    let kind: TimeKind = time.into();

    match kind {
        TimeKind::Euclidean(t) => {
            assert_eq!(t.id(), 1);
            assert_eq!(t.time_scale(), TimeScale::Second);
            assert_eq!(t.time_unit(), 3.00);
        }
        _ => panic!("Expected TimeKind::Euclidean variant"),
    }
}
