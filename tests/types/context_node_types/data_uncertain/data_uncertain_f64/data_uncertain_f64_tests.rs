
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Datable, Identifiable, UncertainFloat64Data};
use deep_causality_uncertain::Uncertain;

#[test]
fn test_new() {
    let id = 1;
    let data = Uncertain::point(42.0);
    let ufd = UncertainFloat64Data::new(id, data.clone());
    assert_eq!(ufd.id(), id);
    assert!((ufd.get_data().sample().unwrap() - 42.0).abs() < f64::EPSILON);
}

#[test]
fn test_id() {
    let id = 42;
    let data = Uncertain::point(1.0);
    let ufd = UncertainFloat64Data::new(id, data);
    assert_eq!(ufd.id(), id);
}

#[test]
fn test_get_data() {
    let id = 1;
    let data = Uncertain::point(std::f64::consts::PI);
    let ufd = UncertainFloat64Data::new(id, data.clone());
    assert!((ufd.get_data().sample().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON);
}

#[test]
fn test_set_data() {
    let id = 1;
    let initial_data = Uncertain::point(1.0);
    let mut ufd = UncertainFloat64Data::new(id, initial_data);
    assert!((ufd.get_data().sample().unwrap() - 1.0).abs() < f64::EPSILON);

    let new_data = Uncertain::point(2.0);
    ufd.set_data(new_data.clone());
    assert!((ufd.get_data().sample().unwrap() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_display() {
    let id = 1;
    let data = Uncertain::normal(0.0, 1.0);
    let ufd = UncertainFloat64Data::new(id, data.clone());
    let display_str = format!("{}", ufd);
    assert!(display_str.contains("UncertainFloat64Data: id: 1"));
    assert!(display_str.contains("data:"));
}
