/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::RayHeight;

#[test]
fn test_ray_height_default() {
    let y: RayHeight<f64> = Default::default();
    assert_eq!(y.value(), 0.0);
}

#[test]
fn test_ray_height_new() {
    let y = RayHeight::<f64>::new(10.0).unwrap();
    assert_eq!(y.value(), 10.0);
}

#[test]
fn test_ray_height_into_f64() {
    let v: f64 = RayHeight::<f64>::new(0.02).unwrap().into();
    assert!((v - 0.02).abs() < 1e-10);
}
