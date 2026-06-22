/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::FocalLength;

#[test]
fn test_focal_length() {
    let f = FocalLength::<f64>::new(-0.5).unwrap();
    assert_eq!(f.value(), -0.5);
}

#[test]
fn test_focal_length_default() {
    let f: FocalLength<f64> = Default::default();
    assert_eq!(f.value(), 0.0);
}

#[test]
fn test_focal_length_into_f64() {
    let v: f64 = FocalLength::<f64>::new(-0.5).unwrap().into();
    assert!((v - (-0.5)).abs() < 1e-10);
}
