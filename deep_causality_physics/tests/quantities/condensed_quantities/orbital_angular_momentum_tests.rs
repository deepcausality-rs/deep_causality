/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::OrbitalAngularMomentum;

#[test]
fn test_orbital_angular_momentum() {
    let oam = OrbitalAngularMomentum::<f64>::new(-3.0).unwrap();
    assert_eq!(oam.value(), -3.0);
    let val: f64 = oam.into();
    assert_eq!(val, -3.0);
}

#[test]
fn test_orbital_angular_momentum_default() {
    let oam: OrbitalAngularMomentum<f64> = Default::default();
    assert_eq!(oam.value(), 0.0);
}
