/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Dual, Euler, Integrator, Rk4};

const E: f64 = std::f64::consts::E;

#[test]
fn test_euler_integrates_exponential() {
    // y' = y, y(0) = 1 → e at t = 1.
    let y = Euler.integrate(1.0_f64, 1e-4, 10_000, &|y: &f64| *y);
    assert!((y - E).abs() < 1e-3);
}

#[test]
fn test_euler_error_is_first_order() {
    let err =
        |dt: f64, steps: usize| (Euler.integrate(1.0_f64, dt, steps, &|y: &f64| *y) - E).abs();
    let e1 = err(1e-2, 100); // dt, to t = 1
    let e2 = err(5e-3, 200); // dt/2, to t = 1
    assert!(e2 < e1);
    let ratio = e1 / e2; // ≈ 2 for first order
    assert!((1.6..=2.4).contains(&ratio), "first-order ratio {ratio}");
}

#[test]
fn test_rk4_is_fourth_order() {
    let err = |dt: f64, steps: usize| (Rk4.integrate(1.0_f64, dt, steps, &|y: &f64| *y) - E).abs();
    let e_dt = err(0.1, 10);
    let e_half = err(0.05, 20);
    let ratio = e_dt / e_half; // ≈ 16 for fourth order
    assert!((10.0..=22.0).contains(&ratio), "fourth-order ratio {ratio}");
}

#[test]
fn test_rk4_beats_euler_at_fixed_dt() {
    let euler = (Euler.integrate(1.0_f64, 0.1, 10, &|y: &f64| *y) - E).abs();
    let rk4 = (Rk4.integrate(1.0_f64, 0.1, 10, &|y: &f64| *y) - E).abs();
    assert!(rk4 < euler / 1000.0);
}

#[test]
fn test_accuracy_is_a_type_swap() {
    // Identical model and call shape; only the integrator value differs.
    let rate = |y: &f64| *y;
    let by_euler = Euler.integrate(1.0_f64, 0.01, 100, &rate);
    let by_rk4 = Rk4.integrate(1.0_f64, 0.01, 100, &rate);
    assert!((by_rk4 - E).abs() < (by_euler - E).abs());
}

#[test]
fn test_integrator_handles_non_scalar_module_state() {
    // `Dual<f64>` is a two-component module over f64; the same integrator marches it
    // with no type-specific code. y' = y grows both channels as eᵗ.
    let y = Rk4.integrate(Dual::new(1.0_f64, 2.0), 0.1, 10, &|s: &Dual<f64>| *s);
    assert!((y.value() - E).abs() < 1e-4);
    assert!((y.derivative() - 2.0 * E).abs() < 1e-4);
}

#[test]
fn test_single_step_matches_euler_formula() {
    // One Euler step of y' = y² at y = 3, dt = 0.5: y + y²·dt = 3 + 9·0.5 = 7.5.
    let next = Euler.step(&3.0_f64, 0.5, &|y: &f64| y * y);
    assert!((next - 7.5).abs() < 1e-12);
}

#[test]
fn test_rk4_conserves_harmonic_oscillator_energy() {
    // Encode the SHM state (x, v) as a `Dual`'s two channels (re, du). The integrator
    // uses only Add + scalar Mul, so it marches the pair as a 2-module. With x' = v,
    // v' = −x the energy x² + v² is invariant, and RK4 holds it over many steps.
    let rate = |s: &Dual<f64>| Dual::new(s.derivative(), -s.value());
    let mut state = Dual::new(1.0_f64, 0.0); // x = 1, v = 0
    let energy0 = state.value().powi(2) + state.derivative().powi(2);
    for _ in 0..1000 {
        state = Rk4.step(&state, 0.01, &rate);
    }
    let energy = state.value().powi(2) + state.derivative().powi(2);
    assert!(
        (energy - energy0).abs() < 1e-6,
        "energy drift {}",
        (energy - energy0).abs()
    );
}
