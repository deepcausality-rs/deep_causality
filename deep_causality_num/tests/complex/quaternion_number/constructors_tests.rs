/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;

#[test]
fn test_new() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q.w, 1.0);
    assert_eq!(q.x, 2.0);
    assert_eq!(q.y, 3.0);
    assert_eq!(q.z, 4.0);
}

#[test]
fn test_identity() {
    let q = Quaternion::<f64>::identity();
    assert_eq!(q.w, 1.0);
    assert_eq!(q.x, 0.0);
    assert_eq!(q.y, 0.0);
    assert_eq!(q.z, 0.0);
}

#[test]
fn test_from_axis_angle() {
    let axis = [1.0, 0.0, 0.0];
    let angle = std::f64::consts::PI / 2.0; // 90 degrees around X axis
    let q = Quaternion::from_axis_angle(axis, angle);

    // Expected quaternion for 90 deg rotation around X: (cos(45), sin(45), 0, 0)
    let expected_w = (angle / 2.0).cos();
    let expected_x = (angle / 2.0).sin();

    const EPSILON: f64 = 1e-9;
    assert!((q.w - expected_w).abs() < EPSILON);
    assert!((q.x - expected_x).abs() < EPSILON);
    assert!((q.y - 0.0).abs() < EPSILON);
    assert!((q.z - 0.0).abs() < EPSILON);

    // Test with zero length axis
    let zero_axis = [0.0, 0.0, 0.0];
    let q_zero_axis = Quaternion::from_axis_angle(zero_axis, angle);
    assert_eq!(q_zero_axis, Quaternion::<f64>::identity());
}

#[test]
fn test_from_euler_angles() {
    let roll = std::f64::consts::PI / 2.0; // 90 degrees
    let pitch = 0.0;
    let yaw = 0.0;
    let q = Quaternion::from_euler_angles(roll, pitch, yaw);

    // Expected for roll 90 deg: (cos(45), sin(45), 0, 0)
    let expected_w = (roll / 2.0).cos();
    let expected_x = (roll / 2.0).sin();

    const EPSILON: f64 = 1e-9;
    assert!((q.w - expected_w).abs() < EPSILON);
    assert!((q.x - expected_x).abs() < EPSILON);
    assert!((q.y - 0.0).abs() < EPSILON);
    assert!((q.z - 0.0).abs() < EPSILON);
}
