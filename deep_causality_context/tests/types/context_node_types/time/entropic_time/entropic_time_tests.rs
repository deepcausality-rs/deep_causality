/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_creation() {
    let t1 = EntropicTime::new(1, 0);
    let t2 = EntropicTime::new(2, 42);

    assert_eq!(t1.id(), 1);
    assert_eq!(t1.time_unit(), 0);
    assert_eq!(t2.id(), 2);
    assert_eq!(t2.time_unit(), 42);
}

#[test]
fn test_progression() {
    let t1 = EntropicTime::new(1, 0); // initial entropy
    let t2 = EntropicTime::new(2, 1); // one step forward

    assert!(t1.time_unit() < t2.time_unit());
}

#[test]
fn test_temporal_trait_behavior() {
    let t = EntropicTime::new(3, 100);
    assert_eq!(t.time_scale(), TimeScale::NoScale);
    assert_eq!(t.time_unit(), 100);
}

#[test]
fn test_scalar_projector_trait() {
    let t = EntropicTime::new(4, 77);
    let projected = t.project();
    assert_eq!(projected, 77);
}

#[test]
fn test_display_trait_output() {
    let t = EntropicTime::new(5, 99);
    let output = format!("{t}");

    assert!(output.contains("EntropicTime"));
    assert!(output.contains("id: 5"));
    assert!(output.contains("tick_scale: NoScale"));
    assert!(output.contains("tick_unit: 99"));
}

#[test]
fn test_from_entropic_time_to_time_kind() {
    let time = EntropicTime::new(7, 1);
    let kind: TimeKind = time.into();

    match kind {
        TimeKind::Entropic(t) => {
            assert_eq!(t.id(), 7);
            assert_eq!(t.time_unit(), 1);
        }
        _ => panic!("Expected TimeKind::Entropic variant"),
    }
}
