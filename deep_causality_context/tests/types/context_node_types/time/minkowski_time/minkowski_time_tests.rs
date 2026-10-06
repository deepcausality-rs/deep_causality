/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{Identifiable, MinkowskiTime, ScalarProjector, Temporal, TimeScale};

#[test]
fn test_construction() {
    let t = MinkowskiTime::new(1, TimeScale::Second, 42.0);
    assert_eq!(t.id(), 1);
    assert_eq!(t.time_scale(), TimeScale::Second);
    assert_eq!(t.time_unit(), 42.0);
}

#[test]
fn test_identifiable_trait() {
    let t = MinkowskiTime::new(99, TimeScale::Millisecond, 12.5);
    assert_eq!(t.id(), 99);
}

#[test]
fn test_temporal_trait() {
    let t = MinkowskiTime::<FloatType>::new(5, TimeScale::Nanoseconds, 0.000_001);
    assert_eq!(t.time_scale(), TimeScale::Nanoseconds);
    assert!((t.time_unit() - 0.000_001).abs() < f64::EPSILON);
}

#[test]
fn test_scalar_projector_trait() {
    let t = MinkowskiTime::<FloatType>::new(2, TimeScale::Second, 3.00);
    assert!((t.project() - 3.00).abs() < f64::EPSILON);
}

#[test]
fn test_display_trait() {
    let t = MinkowskiTime::new(7, TimeScale::Second, 123.456);
    let s = format!("{t}");
    assert!(s.contains("MinkowskiTime"));
    assert!(s.contains("id: 7"));
    assert!(s.contains("time_scale: Second"));
    assert!(s.contains("123.456"));
}

#[test]
fn test_partial_eq() {
    let t1 = MinkowskiTime::new(1, TimeScale::Second, 100.0);
    let t2 = MinkowskiTime::new(1, TimeScale::Second, 100.0);
    let t3 = MinkowskiTime::new(2, TimeScale::Second, 100.0);
    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn test_copy() {
    let t1 = MinkowskiTime::new(42, TimeScale::Millisecond, 1.23);
    let t2 = t1;
    let t3 = t1;
    assert_eq!(t1, t2);
    assert_eq!(t1, t3);
}

use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_from_minkowski_time_to_time_kind() {
    let time = MinkowskiTime::new(42, TimeScale::Second, 3.00);
    let kind: TimeKind<FloatType> = time.into();

    match kind {
        TimeKind::Minkowski(t) => {
            assert_eq!(t.id(), 42);
            assert_eq!(t.time_scale(), TimeScale::Second);
            assert_eq!(t.time_unit(), 3.00);
        }
        _ => panic!("Expected TimeKind::Minkowski variant"),
    }
}
