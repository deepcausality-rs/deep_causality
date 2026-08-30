/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{EndoArrow, Euler};
use deep_causality_haft::Arrow;

const E: f64 = std::f64::consts::E;

#[test]
fn test_euler_marches_exponential() {
    // y' = y, y(0) = 1 → e at t = 1.
    let step = Euler::new(1e-4_f64, |y: &f64| *y);
    let y = step.iterate_n(1.0, 10_000);
    assert!((y - E).abs() < 1e-3);
}

#[test]
fn test_euler_is_first_order() {
    let err = |dt: f64, n: usize| (Euler::new(dt, |y: &f64| *y).iterate_n(1.0, n) - E).abs();
    let e1 = err(1e-2, 100); // to t = 1
    let e2 = err(5e-3, 200); // dt/2, to t = 1
    assert!(e2 < e1);
    let ratio = e1 / e2; // ≈ 2 for first order
    assert!((1.6..=2.4).contains(&ratio), "ratio {ratio}");
}

#[test]
fn test_euler_single_step_formula() {
    // One step of y' = y² at y = 3, dt = 0.5: 3 + 9·0.5 = 7.5.
    let step = Euler::new(0.5_f64, |y: &f64| y * y);
    assert!((step.run(3.0) - 7.5).abs() < 1e-12);
}
