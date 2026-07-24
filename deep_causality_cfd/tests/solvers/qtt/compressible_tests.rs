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
fn run_rejects_non_finite_momentum() {
    // A positive finite density with a non-finite momentum (NaN) gives a NaN pressure
    // `p = (γ−1)(E − ½m²/ρ)`. Since `close-qtt-solver-envelope`, the pressure guard inside
    // `flux_and_speed` catches this at the **root cause** — a non-finite pressure at a named cell —
    // rather than three steps downstream at the aggregate `s_max` wave-speed check, which this input
    // used to reach. The state is still rejected; the diagnostic is now more specific.
    //
    // The `s_max <= 0 || !finite` guard in `run` is retained as defence in depth (a future flux
    // variant could produce a degenerate wave speed with valid pressure), but with every returned
    // cell now guaranteed `p > 0`, `s_max = max(|u|+c) ≥ c > 0`, so it is no longer the first line
    // for this input.
    let l = 4usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let state0 = (vec![1.0; n], vec![f64::NAN; n], vec![2.5; n]);
    let err = solver.run(&state0, 0.05).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "a NaN momentum yields a NaN pressure, refused at the pressure guard: {err:?}"
    );
}

// --- Pressure positivity guard (close-qtt-solver-envelope, items 12 / 12b) ------------------------
//
// The ideal-gas EOS is hyperbolic only for p > 0. A state with E < ½m²/ρ yields p < 0 while ρ stays
// positive, so the density guard passes and the pressure guard must fire. Before this change the
// flux carried the unfloored p while only the wave speed was floored — and that floor,
// `from_f64(1e-300)`, was exactly 0.0 at f32 (an infallible lossy cast returns Some(0.0)), so it
// vanished in a supported precision. The fix rejects instead of flooring, which is a comparison and
// therefore identical at every precision.

use deep_causality_tensor::Truncation as Trunc;

/// Build a one-cell-bad 1-D Euler state at precision `R` and report whether one march step rejects
/// it. `ρ = 1, m = 2 (u = 2), E = 1` ⇒ `p = (γ−1)(E − ½m²/ρ) = 0.4·(1 − 2) = −0.4 < 0`.
fn euler_rejects_non_hyperbolic<R>() -> bool
where
    R: deep_causality_cfd::CfdScalar + deep_causality_algebra::ConjugateScalar<Real = R>,
{
    let l = 4usize;
    let n = 1usize << l;
    let f = |x: f64| R::from_f64(x).unwrap();
    let dx = f(1.0 / n as f64);
    let tr = Trunc::<R>::by_tol(f(1e-6)).unwrap();
    let solver = CompressibleEuler1d::<R>::new(l, dx, f(1.4), f(0.4), tr).unwrap();

    let mut rho = vec![f(1.0); n];
    let mut mom = vec![f(2.0); n];
    let mut energy = vec![f(1.0); n];
    // Keep every other cell hyperbolic so only the seeded cell is non-hyperbolic.
    for i in 0..n {
        if i != n / 2 {
            rho[i] = f(1.0);
            mom[i] = f(0.0);
            energy[i] = f(2.5); // p = 0.4·2.5 = 1.0 > 0
        }
    }
    solver.run(&(rho, mom, energy), f(0.05)).is_err()
}

#[test]
fn a_non_hyperbolic_state_is_rejected() {
    assert!(
        euler_rejects_non_hyperbolic::<f64>(),
        "a cell with p < 0 must be refused, not floored into the flux"
    );
}

#[test]
fn the_pressure_guard_trips_identically_at_f32_and_f64() {
    // The precision-parity scenario: the same non-hyperbolic state must be judged the same way at
    // both precisions. This is the regression test for the `1e-300`-becomes-0.0-at-f32 trap.
    let at_f64 = euler_rejects_non_hyperbolic::<f64>();
    let at_f32 = euler_rejects_non_hyperbolic::<f32>();
    assert_eq!(
        at_f64, at_f32,
        "the guard must trip identically at f32 and f64 (f64={at_f64}, f32={at_f32})"
    );
    assert!(
        at_f32,
        "and it must actually trip, not agree by both passing"
    );
}

#[test]
fn a_valid_state_is_unaffected_by_the_guard() {
    // The guard is inert on the happy path: removing the wave-speed floor is bit-identical for p > 0
    // (p_floor == p there), so a valid non-uniform state still marches to a finite result.
    let l = 5usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let solver = CompressibleEuler1d::<f64>::new(l, dx, 1.4, 0.4, trunc()).unwrap();
    let (rho, u, p) = (1.0, 0.2, 1.0);
    let m = rho * u;
    let e = p / (1.4 - 1.0) + 0.5 * rho * u * u;
    let state = (vec![rho; n], vec![m; n], vec![e; n]);
    let (rf, _mf, ef) = solver.run(&state, 0.05).unwrap();
    assert!(rf.iter().all(|v| v.is_finite()) && ef.iter().all(|v| v.is_finite()));
}
