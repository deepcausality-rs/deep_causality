/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::RayAngle;

#[test]
fn test_ray_angle_default() {
    let theta: RayAngle<f64> = Default::default();
    assert_eq!(theta.value(), 0.0);
}

#[test]
fn test_ray_angle_new() {
    let a = RayAngle::<f64>::new(0.5).unwrap();
    assert_eq!(a.value(), 0.5);
}

#[test]
fn test_ray_angle_into_f64() {
    let v: f64 = RayAngle::<f64>::new(0.1).unwrap().into();
    assert!((v - 0.1).abs() < 1e-10);
}
