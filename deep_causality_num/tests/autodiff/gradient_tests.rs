/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Real, directional_derivative, gradient};

const TOL: f64 = 1e-9;

#[test]
fn test_gradient_norm_squared() {
    // f(x, y) = x² + y² → ∇f(3, 4) = [6, 8]
    let g = gradient(|p| p[0] * p[0] + p[1] * p[1], &[3.0_f64, 4.0]);
    assert_eq!(g, [6.0, 8.0]);
}

#[test]
fn test_gradient_trig_field() {
    // f(x, y) = sin(x)·cos(y) → ∇f = [cos(x)cos(y), −sin(x)sin(y)]
    let x0 = 0.5_f64;
    let y0 = 1.1_f64;
    let g = gradient(|p| p[0].sin() * p[1].cos(), &[x0, y0]);
    assert!((g[0] - x0.cos() * y0.cos()).abs() < TOL);
    assert!((g[1] - (-x0.sin() * y0.sin())).abs() < TOL);
}

#[test]
fn test_gradient_three_inputs() {
    // f(x, y, z) = x·y·z → ∇f = [yz, xz, xy] at (2, 3, 4)
    let g = gradient(|p| p[0] * p[1] * p[2], &[2.0_f64, 3.0, 4.0]);
    assert_eq!(g, [12.0, 8.0, 6.0]);
}

#[test]
fn test_directional_derivative_equals_grad_dot_dir() {
    let x = [2.0_f64, 3.0];
    let dir = [1.0_f64, 1.0];
    let g = gradient(|p| p[0] * p[1], &x);
    let dd = directional_derivative(|p| p[0] * p[1], &x, &dir);
    let want = g[0] * dir[0] + g[1] * dir[1];
    assert!((dd - want).abs() < TOL);
}

#[test]
fn test_directional_derivative_non_unit_direction() {
    // f(x, y) = x² + y², at (1, 1) along (2, 0): ∇f·dir = 2·2 + 2·0 = 4
    let dd = directional_derivative(|p| p[0] * p[0] + p[1] * p[1], &[1.0_f64, 1.0], &[2.0, 0.0]);
    assert!((dd - 4.0).abs() < TOL);
}

#[test]
fn test_gradient_is_precision_generic_f32() {
    let g = gradient(|p| p[0] * p[0] + p[1] * p[1], &[3.0_f32, 4.0]);
    assert_eq!(g, [6.0_f32, 8.0]);
}
