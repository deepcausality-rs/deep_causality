/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 2: the 1-D compressible Euler QTT marcher — conservation, free-stream preservation,
//! and the ideal-gas EOS. (The Sod exact-Riemann gate is the `qtt_sod` verification example.)

use deep_causality_cfd::{CompressibleEuler1d, ideal_gas_pressure};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;

fn trunc() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

#[test]
fn eos_recovers_pressure() {
    // p = (γ−1)(E − ½ m²/ρ). For ρ=1, u=0.1 (m=0.1), p=1: E = p/(γ−1) + ½ρu² = 2.5 + 0.005.
    let gamma = 1.4f64;
    let (rho, u, p) = (1.0f64, 0.1f64, 1.0f64);
    let m = rho * u;
    let e = p / (gamma - 1.0) + 0.5 * rho * u * u;
    let recovered = ideal_gas_pressure(rho, m, e, gamma);
    assert!((recovered - p).abs() < 1e-12, "EOS pressure = {recovered}");
}

#[test]
fn free_stream_preserved() {
    // A uniform state is a fixed point: ∂ₓF(const) = 0 and ∂²ₓ(const) = 0.
    let l = 6usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let (rho, u, p) = (1.2, 0.3, 0.8);
    let m = rho * u;
    let e = p / (1.4 - 1.0) + 0.5 * rho * u * u;
    let state0 = (vec![rho; n], vec![m; n], vec![e; n]);
    let (rf, mf, ef) = solver.run(&state0, 0.1).unwrap();
    for i in 0..n {
        assert!((rf[i] - rho).abs() < 1e-9, "ρ drifted at {i}: {}", rf[i]);
        assert!((mf[i] - m).abs() < 1e-9, "ρu drifted at {i}: {}", mf[i]);
        assert!((ef[i] - e).abs() < 1e-9, "ρE drifted at {i}: {}", ef[i]);
    }
}

#[test]
fn conserves_mass_momentum_energy() {
    // On a periodic domain with no sources, ∫ρ, ∫ρu, ∫ρE are conserved to the rounding floor (the
    // conservative central flux difference and the Rusanov dissipation both sum to zero periodically).
    let l = 7usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let mut rho0 = vec![0.0; n];
    let mut m0 = vec![0.0; n];
    let mut e0 = vec![0.0; n];
    for i in 0..n {
        let x = (i as f64 + 0.5) * dx;
        let rho = 1.0 + 0.2 * (TAU * x).sin();
        let u = 0.1;
        let p = 1.0 + 0.1 * (TAU * x).cos();
        rho0[i] = rho;
        m0[i] = rho * u;
        e0[i] = p / (1.4 - 1.0) + 0.5 * rho * u * u;
    }
    let sum = |v: &[f64]| v.iter().sum::<f64>();
    let (s_rho, s_m, s_e) = (sum(&rho0), sum(&m0), sum(&e0));
    let (rf, mf, ef) = solver.run(&(rho0, m0, e0), 0.05).unwrap();
    assert!(
        (sum(&rf) - s_rho).abs() / s_rho < 1e-9,
        "mass not conserved"
    );
    assert!(
        (sum(&mf) - s_m).abs() / s_m.abs() < 1e-8,
        "momentum not conserved"
    );
    assert!((sum(&ef) - s_e).abs() / s_e < 1e-9, "energy not conserved");
}

#[test]
fn gamma_getter_returns_ratio_of_specific_heats() {
    // The `gamma()` accessor returns the ratio passed to `new`.
    let l = 4usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    assert!((solver.gamma() - 1.4).abs() < 1e-15, "gamma getter");
}

#[test]
fn run_rejects_wrong_length_state() {
    // `run` guards each of the three component buffers against a length ≠ 2^L; a short ρ buffer must
    // surface a `DimensionMismatch`.
    let l = 4usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    // ρ is one cell short of 2^L; ρu and ρE are the right length.
    let state0 = (vec![1.0; n - 1], vec![0.1; n], vec![2.5; n]);
    let err = solver.run(&state0, 0.05).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-length state must be a DimensionMismatch: {err:?}"
    );
}

#[test]
fn run_rejects_non_positive_density() {
    // The flux/EOS enforces the positivity invariant: a negative-density cell in the initial state must
    // surface a `PhysicalInvariantBroken` on the first flux evaluation.
    let l = 4usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let mut rho = vec![1.0; n];
    rho[3] = -0.5; // non-physical density
    let state0 = (rho, vec![0.0; n], vec![2.5; n]);
    let err = solver.run(&state0, 0.05).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "non-positive density must break the positivity invariant: {err:?}"
    );
}

#[test]
fn run_rejects_non_finite_wave_speed() {
    // With a positive, finite density but a non-finite momentum (NaN), the velocity and hence every
    // wave-speed candidate is NaN, so `s_max` never rises above zero and the marcher trips the
    // `NumericalInstability` non-physical-wave-speed guard.
    let l = 4usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let state0 = (vec![1.0; n], vec![f64::NAN; n], vec![2.5; n]);
    let err = solver.run(&state0, 0.05).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::NumericalInstability(_)),
        "a non-finite wave speed must be a NumericalInstability: {err:?}"
    );
}
