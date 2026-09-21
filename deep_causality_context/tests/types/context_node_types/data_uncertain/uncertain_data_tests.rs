/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{Datable, Identifiable, UncertainData};
use deep_causality_core::FloatType;
use deep_causality_uncertain::Uncertain;

#[test]
fn test_new() {
    let id = 1;
    let data = Uncertain::<f64>::point(42.0);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    assert_eq!(ufd.id(), id);
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 42.0).abs() < f64::EPSILON);
}

#[test]
fn test_id() {
    let id = 42;
    let data = Uncertain::<f64>::point(1.0);
    let ufd = UncertainData::<FloatType>::new(id, data);
    assert_eq!(ufd.id(), id);
}

#[test]
fn test_get_data() {
    let id = 1;
    let data = Uncertain::<f64>::point(std::f64::consts::PI);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    assert!(
        (ufd.get_data().sample_from_entropy().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON
    );
}

#[test]
fn test_set_data() {
    let id = 1;
    let initial_data = Uncertain::<f64>::point(1.0);
    let mut ufd = UncertainData::<FloatType>::new(id, initial_data);
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 1.0).abs() < f64::EPSILON);

    let new_data = Uncertain::<f64>::point(2.0);
    ufd.set_data(new_data.clone());
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_display() {
    let id = 1;
    let data = Uncertain::normal(0.0, 1.0);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    let display_str = format!("{}", ufd);
    // A generic `Display` must name the type rather than one instantiation of it — printing a
    // scalar-specific name for an `UncertainData<f32>` would be a lie.
    assert!(display_str.contains("UncertainData: id: 1"));
    assert!(display_str.contains("data:"));
}
