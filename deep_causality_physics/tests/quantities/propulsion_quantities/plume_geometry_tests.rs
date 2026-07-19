/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Length, PlumeGeometry};

fn sample() -> PlumeGeometry<f64> {
    PlumeGeometry::new(
        Length::new(0.043).unwrap(),
        Length::new(0.13).unwrap(),
        Length::new(0.12).unwrap(),
    )
}

#[test]
fn test_plume_geometry_getters() {
    let g = sample();
    assert_eq!(g.max_radius().value(), 0.043);
    assert_eq!(g.penetration_length().value(), 0.13);
    assert_eq!(g.terminal_shock_standoff().value(), 0.12);
}

#[test]
fn test_plume_geometry_default() {
    let g: PlumeGeometry<f64> = Default::default();
    assert_eq!(g.max_radius().value(), 0.0);
    assert_eq!(g.penetration_length().value(), 0.0);
    assert_eq!(g.terminal_shock_standoff().value(), 0.0);
}

#[test]
fn test_plume_geometry_clone_eq() {
    let g = sample();
    let h = g;
    assert_eq!(g, h);
}
