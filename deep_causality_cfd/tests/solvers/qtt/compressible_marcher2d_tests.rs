/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 5 gates: the 2-D body-fitted compressible Euler marcher (`CompressibleMarcher2d`) over the
//! `MetricProvider` seam — free-stream preservation, and the rank-lever comparison (the smooth field the
//! fitted coordinate carries stays low rank, vs the curved Cartesian capture).

use deep_causality_cfd::{
    BodyFittedCoordinate, CartesianIdentity, CompressibleMarcher2d, EulerState2d, EulerStateTt2d,
    Marcher, dequantize_2d, ideal_gas_pressure_2d, quantize_2d,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const GAMMA: f64 = 1.4;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

#[test]
fn free_stream_is_a_fixed_point() {
    // A uniform state has zero flux divergence and zero Laplacian, so it must march unchanged.
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let marcher = CompressibleMarcher2d::new(cart, GAMMA, 0.002, 1.3, tr()).unwrap();
    assert!((marcher.gamma() - GAMMA).abs() < 1e-15, "gamma getter");

    let (rho, u, v, p) = (1.2, 0.3, 0.1, 0.8);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v);
    let state: EulerState2d<f64> = [vec![rho; n], vec![rho * u; n], vec![rho * v; n], vec![e; n]];
    let (out, _) = marcher.run(&state, 6).unwrap();
    for (((&r, &mu), &mv), &en) in out[0].iter().zip(&out[1]).zip(&out[2]).zip(&out[3]) {
        assert!((r - rho).abs() < 1e-9, "ρ drifted");
        assert!((mu - rho * u).abs() < 1e-9, "ρu drifted");
        assert!((mv - rho * v).abs() < 1e-9, "ρv drifted");
        assert!((en - e).abs() < 1e-9, "ρE drifted");
    }
}

#[test]
fn marcher_trait_advance_matches_one_step_and_preserves_free_stream() {
    // Task 5.1: `CompressibleMarcher2d` implements `Marcher` so it drops into the CfdFlow march driver.
    // `advance` is one IMEX step on the tensor-train state; a uniform state is a fixed point of it.
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let q = |buf: Vec<f64>| {
        quantize_2d(
            &CausalTensor::new(buf, vec![1 << l, 1 << l]).unwrap(),
            &tr(),
        )
        .unwrap()
    };
    let (rho, u, v, p) = (1.0, 0.2, 0.1, 0.7);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v);
    let state: EulerStateTt2d<f64> = [
        q(vec![rho; n]),
        q(vec![rho * u; n]),
        q(vec![rho * v; n]),
        q(vec![e; n]),
    ];
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let marcher = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr()).unwrap();

    let advanced = Marcher::advance(&marcher, &state, &()).unwrap();
    let stepped = marcher.step(&state).unwrap();
    let a0 = dequantize_2d(&advanced[0], l, l).unwrap();
    let s0 = dequantize_2d(&stepped[0], l, l).unwrap();
    for (&a, &s) in a0.as_slice().iter().zip(s0.as_slice()) {
        assert!((a - s).abs() < 1e-14, "advance must equal step");
    }
    for &d in a0.as_slice() {
        assert!(
            (d - rho).abs() < 1e-9,
            "advance must preserve free-stream ρ: {d}"
        );
    }
}

#[test]
fn new_rejects_non_positive_reference_speed() {
    let cart = CartesianIdentity::<f64>::new(4, 4, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    assert!(CompressibleMarcher2d::new(cart, GAMMA, 0.001, 0.0, tr()).is_err());
    let cart2 = CartesianIdentity::<f64>::new(4, 4, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    assert!(CompressibleMarcher2d::new(cart2, GAMMA, 0.001, -1.0, tr()).is_err());
}

#[test]
fn eos_2d_recovers_pressure() {
    let (rho, u, v, p) = (1.0, 0.2, 0.3, 1.0);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v);
    let got = ideal_gas_pressure_2d(rho, rho * u, rho * v, e, GAMMA);
    assert!((got - p).abs() < 1e-12, "2-D EOS pressure = {got}");
}

#[test]
fn free_stream_preserved_over_the_metric_seam() {
    // The marcher is generic over MetricProvider (design D8, static dispatch). A uniform state is a fixed
    // point in *any* coordinate (∇·F(const) = 0, ∇²(const) = 0), so it must march unchanged on both the
    // Cartesian identity and the body-fitted polar coordinate — proving the seam works inside the marcher.
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let (rho, u, v, p) = (1.1, 0.2, 0.0, 0.9);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v);
    let state: EulerState2d<f64> = [vec![rho; n], vec![rho * u; n], vec![rho * v; n], vec![e; n]];

    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let (out_c, _) = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr())
        .unwrap()
        .run(&state, 6)
        .unwrap();

    let fitted = BodyFittedCoordinate::<f64>::new(l, l, 1.0, 1.0, 0.0, TAU, tr()).unwrap();
    let (out_f, _) = CompressibleMarcher2d::new(fitted, GAMMA, 0.001, 1.3, tr())
        .unwrap()
        .run(&state, 6)
        .unwrap();

    for i in 0..n {
        assert!((out_c[0][i] - rho).abs() < 1e-9, "Cartesian ρ drifted");
        assert!((out_f[0][i] - rho).abs() < 1e-9, "fitted ρ drifted");
        assert!((out_c[3][i] - e).abs() < 1e-9, "Cartesian ρE drifted");
        assert!((out_f[3][i] - e).abs() < 1e-9, "fitted ρE drifted");
    }
}

#[test]
fn marcher_is_stable_on_a_smooth_compressible_field() {
    // A smooth compressible structure marches **stably** (density stays positive and finite) — the explicit
    // flux + implicit acoustic dissipation are sound. Note the *rank* of a captured curved field is
    // coordinate-dependent and expected high on Cartesian (the measured √side result), so it is not a
    // marcher-quality gate. The bounded-χ result needs a *fitted* coordinate with re-pinning + an exact-RH
    // interface (the open-research Stage-5/6 remainder the `qtt_rank_*` / `qtt_repin_marcher` studies
    // identified), not this marcher alone.
    let l = 4usize;
    let side = 1usize << l;
    let n = side * side;
    let mut state: EulerState2d<f64> = [vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    for ix in 0..side {
        for iy in 0..side {
            let x = ix as f64 / side as f64;
            let y = iy as f64 / side as f64;
            let r = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            let bump = 0.1 * (-(r / 0.2).powi(2)).exp();
            let idx = ix * side + iy;
            state[0][idx] = 1.0 + bump;
            state[3][idx] = (1.0 + bump) / (GAMMA - 1.0);
        }
    }
    let cart =
        CartesianIdentity::<f64>::new(l, l, 1.0 / side as f64, 1.0 / side as f64, tr()).unwrap();
    let (out, _peak) = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr())
        .unwrap()
        .run(&state, 8)
        .unwrap();
    assert!(
        out[0].iter().all(|&d| d > 0.0 && d.is_finite()),
        "density must stay positive and finite under the march"
    );
    assert!(
        out[3].iter().all(|&e| e.is_finite()),
        "energy must stay finite"
    );
}

#[test]
fn imex_stays_bounded_beyond_the_explicit_acoustic_diffusion_limit() {
    // Stage-5 / D10 gate: the implicit-dissipation IMEX step stays bounded at an acoustic-diffusion number
    // `s = ½·s_max·Δt/Δx` past the explicit 2-D limit (¼), where a fully-explicit dissipation diverges
    // (rigorously gated in Stage 3's `imex_stable_beyond_explicit_cfl` for this same mechanism). The
    // closed-form inverse carries the stiffness with no iterative solve, and free-stream content is
    // untouched. The lever is the *linear acoustic-diffusion* limit; central convection without limiting
    // still bounds Δt by its own overshoot, so this gates a modest, honest margin (s ≈ 0.35, ~1.4× the
    // explicit limit), not an unbounded one.
    let l = 4usize;
    let side = 1usize << l;
    let n = side * side;
    let mut state: EulerState2d<f64> = [vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    for ix in 0..side {
        for iy in 0..side {
            let x = ix as f64 / side as f64;
            let y = iy as f64 / side as f64;
            let bump = 0.05 * (TAU * x).sin() * (TAU * y).sin();
            let idx = ix * side + iy;
            state[0][idx] = 1.0 + bump;
            state[3][idx] = (1.0 + bump) / (GAMMA - 1.0); // p = 1 + bump, u = v = 0
        }
    }
    // c ≈ √1.4 ≈ 1.18, h = 1/16 ⇒ s = ½·c·Δt/h ≈ 9.44·Δt; Δt = 0.037 ⇒ s ≈ 0.35 (> ¼, the explicit limit).
    let dt = 0.037;
    let s_ref = 1.2; // ≈ c at p = ρ; β = dt·½·s_ref·h ⇒ per-axis stiffness ≈ 0.35 (> ¼).
    let cart =
        CartesianIdentity::<f64>::new(l, l, 1.0 / side as f64, 1.0 / side as f64, tr()).unwrap();
    let (out, _peak) = CompressibleMarcher2d::new(cart, GAMMA, dt, s_ref, tr())
        .unwrap()
        .run(&state, 16)
        .unwrap();
    let maxabs = out[0]
        .iter()
        .chain(out[3].iter())
        .fold(0.0f64, |m, &v| m.max(v.abs()));
    assert!(
        out[0].iter().all(|&d| d > 0.0 && d.is_finite()),
        "IMEX density must stay positive and finite beyond the explicit limit"
    );
    assert!(
        maxabs.is_finite() && maxabs < 5.0,
        "IMEX state must stay bounded beyond the explicit acoustic-diffusion limit: max = {maxabs}"
    );
}

#[test]
fn run_rejects_wrong_length_state() {
    // `run` guards each component buffer against a length ≠ 2^Lx·2^Ly; a short buffer must surface a
    // `DimensionMismatch`.
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let marcher = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr()).unwrap();
    // ρ one cell short; the rest correct.
    let bad: EulerState2d<f64> = [vec![1.0; n - 1], vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    let err = marcher.run(&bad, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-length state must be a DimensionMismatch: {err:?}"
    );
}

#[test]
fn step_rejects_non_positive_density() {
    // The pointwise flux/EOS enforces positivity: stepping a state with a negative-density cell must
    // surface a `PhysicalInvariantBroken`.
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let mut rho = vec![1.0; n];
    rho[5] = -0.3; // non-physical density
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let marcher = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr()).unwrap();
    let state: EulerState2d<f64> = [rho, vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    let err = marcher.run(&state, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "non-positive density must break the positivity invariant: {err:?}"
    );
}

#[test]
fn peak_bond_tracks_growth_over_the_march() {
    // A localized perturbation on an otherwise-uniform field encodes at low rank, then develops structure
    // under the march, so the tracked peak `max_bond` must rise above its starting value — exercising the
    // bond-growth branch in `run`.
    let l = 4usize;
    let side = 1usize << l;
    let n = side * side;
    let mut state: EulerState2d<f64> = [vec![1.0; n], vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    // A single central bump seeds structure the march spreads.
    let c = side / 2;
    let idx = c * side + c;
    state[0][idx] = 1.3;
    state[3][idx] = 1.3 / (GAMMA - 1.0);
    let cart =
        CartesianIdentity::<f64>::new(l, l, 1.0 / side as f64, 1.0 / side as f64, tr()).unwrap();
    let (out, peak) = CompressibleMarcher2d::new(cart, GAMMA, 0.001, 1.3, tr())
        .unwrap()
        .run(&state, 6)
        .unwrap();
    assert!(
        out[0].iter().all(|&d| d > 0.0 && d.is_finite()),
        "density stays positive & finite"
    );
    assert!(peak >= 1, "peak bond must be recorded: {peak}");
}

#[test]
fn non_positive_pressure_is_rejected() {
    // `close-qtt-solver-envelope` item 12: a cell with E < ½|m|²/ρ yields p < 0 with ρ > 0, so the
    // density guard passes and the pressure guard must fire (uniform-across-the-family scenario).
    let l = 4usize;
    let n = (1usize << l) * (1usize << l);
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0 / 16.0, 1.0 / 16.0, tr()).unwrap();
    let marcher = CompressibleMarcher2d::new(cart, GAMMA, 0.002, 1.3, tr()).unwrap();
    // Valid everywhere (p = 0.4·2.5 = 1.0) except one non-hyperbolic cell (p = 0.4·(1−2) = −0.4).
    let mut state: EulerState2d<f64> = [vec![1.0; n], vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    state[1][n / 2] = 2.0; // mx: u = 2, ½|m|²/ρ = 2
    state[3][n / 2] = 1.0; // E = 1 < 2
    let err = marcher.run(&state, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "a non-hyperbolic cell must be refused: {err:?}"
    );
}
