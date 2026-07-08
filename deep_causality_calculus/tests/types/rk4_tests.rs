/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{EndoArrow, Euler, Rk4};
use deep_causality_num_dual::Dual;

const E: f64 = std::f64::consts::E;

#[test]
fn test_rk4_is_fourth_order() {
    let err = |dt: f64, n: usize| (Rk4::new(dt, |y: &f64| *y).iterate_n(1.0, n) - E).abs();
    let e1 = err(0.1, 10);
    let e2 = err(0.05, 20);
    let ratio = e1 / e2; // ≈ 16 for fourth order
    assert!((10.0..=22.0).contains(&ratio), "ratio {ratio}");
}

#[test]
fn test_rk4_beats_euler_at_fixed_dt() {
    let euler = (Euler::new(0.1_f64, |y: &f64| *y).iterate_n(1.0, 10) - E).abs();
    let rk4 = (Rk4::new(0.1_f64, |y: &f64| *y).iterate_n(1.0, 10) - E).abs();
    assert!(rk4 < euler / 1000.0);
}

#[test]
fn test_accuracy_is_a_type_swap() {
    // Same rate field and call shape; only the stepper differs.
    let by_euler = Euler::new(0.01_f64, |y: &f64| *y).iterate_n(1.0, 100);
    let by_rk4 = Rk4::new(0.01_f64, |y: &f64| *y).iterate_n(1.0, 100);
    assert!((by_rk4 - E).abs() < (by_euler - E).abs());
}

#[test]
fn test_rk4_conserves_harmonic_oscillator_energy() {
    // SHM (x, v) carried in a Dual's two channels: x' = v, v' = −x. Energy x² + v² conserved.
    // The same stepper marches a non-scalar module state.
    let step = Rk4::new(0.01_f64, |s: &Dual<f64>| {
        Dual::new(s.derivative(), -s.value())
    });
    let start = Dual::new(1.0_f64, 0.0); // x = 1, v = 0
    let e0 = start.value().powi(2) + start.derivative().powi(2);
    let end = step.iterate_n(start, 1000);
    let e = end.value().powi(2) + end.derivative().powi(2);
    assert!((e - e0).abs() < 1e-6, "energy drift {}", (e - e0).abs());
}
