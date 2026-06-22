/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::BerryCurvature;

#[test]
fn test_berry_curvature() {
    let bc = BerryCurvature::<f64>::new(1.0).unwrap();
    assert_eq!(bc.value(), 1.0);
    let val: f64 = bc.into();
    assert_eq!(val, 1.0);
}

#[test]
fn test_berry_curvature_default() {
    let bc: BerryCurvature<f64> = Default::default();
    assert_eq!(bc.value(), 0.0);
}
