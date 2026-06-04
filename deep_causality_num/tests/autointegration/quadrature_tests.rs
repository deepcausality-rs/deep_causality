/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Dual, Real, quadrature};
use std::f64::consts::PI;

#[test]
fn test_quadrature_exact_on_cubic() {
    // Simpson is exact through cubics: ∫₀¹ x³ dx = 1/4.
    assert!((quadrature(|x: f64| x * x * x, 0.0, 1.0, 2) - 0.25).abs() < 1e-12);
    assert!((quadrature(|x: f64| x * x * x, 0.0, 1.0, 50) - 0.25).abs() < 1e-12);
}

#[test]
fn test_quadrature_odd_panel_count_is_normalised_even() {
    // n = 3 (odd) → bumped to 4; still exact on a cubic.
    assert!((quadrature(|x: f64| x * x * x, 0.0, 1.0, 3) - 0.25).abs() < 1e-12);
}

#[test]
fn test_quadrature_min_panel_count() {
    // n = 0 and n = 1 → normalised to the minimum of 2 panels.
    assert!((quadrature(|x: f64| x * x * x, 0.0, 1.0, 0) - 0.25).abs() < 1e-12);
    assert!((quadrature(|x: f64| x * x * x, 0.0, 1.0, 1) - 0.25).abs() < 1e-12);
}

#[test]
fn test_quadrature_converges_on_sine() {
    // ∫₀^π sin(x) dx = 2.
    assert!((quadrature(|x: f64| x.sin(), 0.0, PI, 100) - 2.0).abs() < 1e-6);
}

#[test]
fn test_quadrature_precision_generic_f32() {
    assert!((quadrature(|x: f32| x * x * x, 0.0, 1.0, 8) - 0.25).abs() < 1e-6);
}

#[test]
fn test_quadrature_leibniz_bridge_over_dual() {
    // I(θ) = ∫₀¹ sin(θ·x) dx = (1 − cos θ)/θ.
    // dI/dθ = (θ·sin θ − (1 − cos θ)) / θ².
    // Seeding θ as a variable dual yields both in one sweep.
    let theta0 = 1.3_f64;
    let theta = Dual::variable(theta0);
    let i = quadrature(
        |x: Dual<f64>| (x * theta).sin(),
        Dual::constant(0.0),
        Dual::constant(1.0),
        200,
    );
    let want_value = (1.0 - theta0.cos()) / theta0;
    let want_derivative = (theta0 * theta0.sin() - (1.0 - theta0.cos())) / (theta0 * theta0);
    assert!((i.value() - want_value).abs() < 1e-6, "value {}", i.value());
    assert!(
        (i.derivative() - want_derivative).abs() < 1e-6,
        "derivative {}",
        i.derivative()
    );
}
