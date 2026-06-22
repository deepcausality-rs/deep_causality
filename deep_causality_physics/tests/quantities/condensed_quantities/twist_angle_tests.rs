/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::TwistAngle;

#[test]
fn test_twist_angle() {
    let ta = TwistAngle::<f64>::new(1.0); // radians
    assert!(ta.is_ok());

    let deg = TwistAngle::<f64>::from_degrees(180.0);
    assert!((deg.value() - std::f64::consts::PI).abs() < 1e-10);
    assert!((deg.as_degrees() - 180.0).abs() < 1e-10);
}

#[test]
fn test_twist_angle_default() {
    let t: TwistAngle<f64> = TwistAngle::default();
    assert_eq!(t.value(), 0.0);
}

#[test]
fn test_twist_angle_degrees_roundtrip() {
    let t = TwistAngle::<f64>::from_degrees(45.0);
    assert!((t.as_degrees() - 45.0).abs() < 1e-10);
}

#[test]
fn test_twist_angle_into_f64() {
    let ta = TwistAngle::<f64>::new(1.1).unwrap();
    let val: f64 = ta.into();
    assert!((val - 1.1).abs() < 1e-10);
}
