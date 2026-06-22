/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::OpticalPower;

#[test]
fn test_optical_power() {
    let p = OpticalPower::<f64>::new(2.0).unwrap();
    assert_eq!(p.value(), 2.0);
}

#[test]
fn test_optical_power_default() {
    let p: OpticalPower<f64> = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_optical_power_into_f64() {
    let v: f64 = OpticalPower::<f64>::new(2.0).unwrap().into();
    assert!((v - 2.0).abs() < 1e-10);
}
