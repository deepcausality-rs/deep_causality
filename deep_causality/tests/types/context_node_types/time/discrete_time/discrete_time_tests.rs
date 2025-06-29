/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

#[test]
fn test_creation_and_accessors() {
    let dt = DiscreteTime::new(42, TimeScale::Millisecond, 100);

    assert_eq!(dt.id(), 42);
    assert_eq!(dt.time_scale(), TimeScale::Millisecond);
    assert_eq!(dt.time_unit(), 100);
}

#[test]
fn test_display_trait() {
    let dt = DiscreteTime::new(1, TimeScale::Second, 1_000);
    let output = format!("{dt}");

    assert!(output.contains("DiscreteTime"));
    assert!(output.contains("id: 1"));
    assert!(output.contains("tick_scale: Second"));
    assert!(output.contains("tick_unit: 1000"));
}

#[test]
fn test_identifiable_trait() {
    let dt = DiscreteTime::new(7, TimeScale::Microseconds, 999);
    assert_eq!(dt.id(), 7);
}

#[test]
fn test_temporal_trait() {
    let dt = DiscreteTime::new(21, TimeScale::Microseconds, 7);

    assert_eq!(dt.time_scale(), TimeScale::Microseconds);
    assert_eq!(dt.time_unit(), 7);
}

#[test]
fn test_scalar_projector_trait() {
    let dt = DiscreteTime::new(99, TimeScale::Nanoseconds, 12345);
    let scalar = dt.project();

    assert_eq!(scalar, 12345);
}

#[test]
fn test_from_discrete_time_to_time_kind() {
    let time = DiscreteTime::new(42, TimeScale::Second, 123);
    let kind: TimeKind = time.into();

    match kind {
        TimeKind::Discrete(t) => {
            assert_eq!(t.id(), 42);
            assert_eq!(t.time_scale(), TimeScale::Second);
            assert_eq!(t.time_unit(), 123);
        }
        _ => panic!("Expected TimeKind::Discrete variant"),
    }
}
