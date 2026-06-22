/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Activity;

#[test]
fn test_activity_new_valid() {
    let activity = Activity::<f64>::new(3.7e10); // 1 Curie in Becquerels
    assert!(activity.is_ok());
}

#[test]
fn test_activity_new_negative_error() {
    let activity = Activity::<f64>::new(-1.0);
    assert!(activity.is_err());
}

#[test]
fn test_activity_new_unchecked() {
    let activity = Activity::<f64>::new_unchecked(1e6);
    assert!((activity.value() - 1e6).abs() < 1.0);
}

#[test]
fn test_activity_from_f64() {
    let activity = Activity::<f64>::new(500.0).unwrap();
    let val: f64 = activity.into();
    assert!((val - 500.0).abs() < 1e-10);
}

#[test]
fn test_activity_default() {
    let a: Activity<f64> = Activity::default();
    assert_eq!(a.value(), 0.0);
}
