/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 3 gates: the IMEX split-acoustic integrator (`AcousticImex1d`) and its conservation /
//! positivity helpers. The acoustic operator is exercised in isolation (task 3.1), per design D10 and the
//! `studies/qtt_acoustic_precond` result.

use deep_causality_cfd::{
    AcousticImex1d, conservation_round, dequantize, positivity_floor, quantize,
};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

const TAU: f64 = core::f64::consts::TAU;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

fn enc(v: Vec<f64>) -> CausalTensorTrain<f64> {
    let n = v.len();
    quantize(&CausalTensor::new(v, vec![n]).unwrap(), &tr()).unwrap()
}

fn dense(u: &CausalTensorTrain<f64>) -> Vec<f64> {
    dequantize(u).unwrap().as_slice().to_vec()
}

fn maxabs(u: &CausalTensorTrain<f64>) -> f64 {
    dense(u).iter().map(|v| v.abs()).fold(0.0, f64::max)
}

fn sum(u: &CausalTensorTrain<f64>) -> f64 {
    dense(u).iter().sum()
}

/// Smooth `c²(x) = (1 + 0.3·sin 2πx)²` and its mean.
fn c2_smooth(n: usize) -> (Vec<f64>, f64) {
    let c2: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            let c = 1.0 + 0.3 * (TAU * x).sin();
            c * c
        })
        .collect();
    let cbar2 = c2.iter().sum::<f64>() / n as f64;
    (c2, cbar2)
}

/// Build an integrator at a chosen acoustic diffusion number `s = Δt·κ·c̄²/Δx²` (κ = 1).
fn build(l: usize, s: f64) -> (AcousticImex1d<f64>, usize, f64) {
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let (c2, cbar2) = c2_smooth(n);
    let dt = s * dx * dx / cbar2;
    let imex = AcousticImex1d::<f64>::new(l, dx, 1.0, 1.0, dt, &c2, tr()).unwrap();
    (imex, n, dx)
}

#[test]
fn amen_converges_in_isolation() {
    // Task 3.1: the implicit acoustic solve converges on the (split) operator, gated before any coupling.
    let (imex, n, _dx) = build(7, 4.0);
    let u0 = enc((0..n).map(|i| (TAU * i as f64 / n as f64).sin()).collect());
    assert!(
        imex.step(&u0).is_ok(),
        "the AMEn implicit acoustic solve must converge in isolation"
    );
}

#[test]
fn imex_stable_beyond_explicit_cfl() {
    // Gate 3.3: at acoustic diffusion number 1.0 (≫ the 0.5 explicit limit), a fully-explicit control
    // diverges while the IMEX step stays bounded. The instability rides the Nyquist mode (alternating ±).
    let (imex, n, _dx) = build(7, 1.0);
    let u0 = enc((0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            (TAU * x).sin() + 0.2 * if i % 2 == 0 { 1.0 } else { -1.0 }
        })
        .collect());

    let mut explicit = u0.clone();
    let mut imex_state = u0.clone();
    for _ in 0..12 {
        explicit = imex.explicit_step(&explicit).unwrap();
        imex_state = imex.step(&imex_state).unwrap();
    }
    let me = maxabs(&explicit);
    let mi = maxabs(&imex_state);
    assert!(
        me > 10.0,
        "explicit control must diverge beyond the acoustic CFL: max = {me}"
    );
    assert!(
        mi.is_finite() && mi < 3.0,
        "IMEX must stay bounded where explicit diverges: max = {mi}"
    );
}

#[test]
fn conservation_round_preserves_integral() {
    // Task 3.2: rounding drifts the integral; the rank-1 fixup restores it, even under coarse truncation.
    let n = 1usize << 7;
    let u = enc((0..n)
        .map(|i| 1.0 + 0.5 * (TAU * i as f64 / n as f64).sin())
        .collect());
    let s0 = sum(&u);
    let coarse = Truncation::<f64>::by_tol(1e-2).unwrap();
    let rounded = conservation_round(&u, s0, &coarse).unwrap();
    assert!(
        (sum(&rounded) - s0).abs() < 1e-9,
        "conservation_round must restore the carried integral under coarse rounding"
    );
}

#[test]
fn imex_run_conserves_mass() {
    // Task 3.2: no secular drift of ∫u over a long IMEX run with conservation rounding (periodic mass).
    let (imex, n, _dx) = build(7, 1.0);
    let u0 = enc((0..n)
        .map(|i| 1.0 + 0.5 * (TAU * i as f64 / n as f64).sin())
        .collect());
    let s0 = sum(&u0);
    let mut u = u0;
    for _ in 0..200 {
        u = imex.step(&u).unwrap();
        u = conservation_round(&u, s0, &tr()).unwrap();
    }
    assert!(
        (sum(&u) - s0).abs() / s0.abs() < 1e-8,
        "mass must not drift secularly over the run"
    );
}

#[test]
fn positivity_floor_holds_through_a_steep_front() {
    // Task 3.3: a steep positive profile (central advection would undershoot) stays ≥ floor under the
    // positivity limiter.
    let floor = 0.01;
    let (imex, n, _dx) = build(7, 1.0);
    let u0 = enc((0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            0.5 - 0.49 * ((x - 0.5) / 0.02).tanh()
        })
        .collect());

    let mut u = u0;
    let mut worst = f64::INFINITY;
    for _ in 0..60 {
        u = imex.step(&u).unwrap();
        u = positivity_floor(&u, floor, &tr()).unwrap();
        worst = worst.min(dense(&u).iter().cloned().fold(f64::INFINITY, f64::min));
    }
    assert!(
        worst >= floor - 1e-9,
        "the positivity floor must hold through the steep front: min = {worst}"
    );
}
