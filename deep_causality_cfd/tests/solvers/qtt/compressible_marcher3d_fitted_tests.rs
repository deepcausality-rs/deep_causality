/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 6 — the body-fitted 3-D compressible marcher (`CompressibleMarcher3dFitted`).
//!
//! The decisive gate: over `CartesianIdentity3d` the metric-aware marcher reproduces the Cartesian
//! `CompressibleMarcher3d` to round-off (the divergence-via-metric machinery is correct). Plus free-stream
//! preservation on the identity chart, and a stability + bounded-rank gate on the body-fitted shell.

use deep_causality_cfd::{
    BodyFittedCoordinate3d, CartesianIdentity3d, CompressibleMarcher3d,
    CompressibleMarcher3dFitted, EulerState3d, EulerStateTt3d, Marcher, dequantize_3d, quantize_3d,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const GAMMA: f64 = 1.4;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

/// A smooth, positive, non-uniform 3-D Euler state on a `2^l` cubic lattice.
fn smooth_state(l: usize) -> EulerState3d<f64> {
    let n = 1usize << l;
    let tot = n * n * n;
    let (mut rho, mut mx, mut my, mut mz, mut e) = (
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
    );
    let (u, v, w) = (0.1, -0.05, 0.05);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let (x, y, z) = (
                    i as f64 / n as f64,
                    j as f64 / n as f64,
                    k as f64 / n as f64,
                );
                let d = 1.0 + 0.2 * (TAU * x).sin() * (TAU * y).sin() * (TAU * z).sin();
                let p = 1.0;
                rho.push(d);
                mx.push(d * u);
                my.push(d * v);
                mz.push(d * w);
                e.push(p / (GAMMA - 1.0) + 0.5 * d * (u * u + v * v + w * w));
            }
        }
    }
    [rho, mx, my, mz, e]
}

#[test]
fn fitted_over_cartesian_identity_matches_the_cartesian_marcher() {
    let l = 3usize;
    let dx = 1.0 / (1usize << l) as f64;
    let (dt, s_ref) = (0.002, 1.3);
    let state = smooth_state(l);

    let cart = CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, dt, s_ref, tr()).unwrap();
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let fitted = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, dt, s_ref, tr()).unwrap();

    let (out_cart, _) = cart.run(&state, 3).unwrap();
    let (out_fit, _) = fitted.run(&state, 3).unwrap();

    let mut max_diff = 0.0f64;
    for (a, b) in out_cart.iter().zip(out_fit.iter()) {
        for (p, q) in a.iter().zip(b.iter()) {
            max_diff = max_diff.max((p - q).abs());
        }
    }
    // Identical operators applied in the identical order ⇒ bit-for-bit (allow a round-off margin).
    assert!(
        max_diff < 1e-12,
        "fitted-over-identity must reproduce the Cartesian marcher; max diff {max_diff}"
    );
}

#[test]
fn free_stream_is_a_fixed_point_on_the_identity_chart() {
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let (rho, u, v, w, p) = (1.2, 0.3, 0.1, 0.05, 0.8);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v + w * w);
    let state: EulerState3d<f64> = [
        vec![rho; n],
        vec![rho * u; n],
        vec![rho * v; n],
        vec![rho * w; n],
        vec![e; n],
    ];
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let fitted = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.002, 1.3, tr()).unwrap();
    assert!((fitted.gamma() - GAMMA).abs() < 1e-15);
    let (out, _) = fitted.run(&state, 4).unwrap();
    for &d in &out[0] {
        assert!((d - rho).abs() < 1e-9, "ρ drifted: {d}");
    }
    for &en in &out[4] {
        assert!((en - e).abs() < 1e-9, "ρE drifted: {en}");
    }
}

#[test]
fn body_fitted_shell_marches_stably_with_bounded_rank() {
    let l = 3usize;
    let n = 1usize << l;
    let tot = n * n * n;
    // A mild radial density gradient (function of the radial ζ = z-lattice), small uniform velocity.
    let (u, v, w, p) = (0.05, 0.0, 0.0, 1.0);
    let (mut rho, mut mx, mut my, mut mz, mut e) = (
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
    );
    for _i in 0..n {
        for _j in 0..n {
            for k in 0..n {
                let d = 1.0 + 0.1 * (k as f64 / n as f64);
                rho.push(d);
                mx.push(d * u);
                my.push(d * v);
                mz.push(d * w);
                e.push(p / (GAMMA - 1.0) + 0.5 * d * (u * u + v * v + w * w));
            }
        }
    }
    let state: EulerState3d<f64> = [rho, mx, my, mz, e];

    let shell =
        BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr()).unwrap();
    let dx = 1.0 / n as f64;
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, 0.0005, 2.0, tr()).unwrap();

    // Stability gate: density-positivity is enforced inside the flux (an Err on ρ ≤ 0), so a successful
    // run proves the fitted marcher stays stable; density and energy stay finite, and the bond stays
    // within the representable full-rank ceiling (2^(3l/2) = 16 on 8³). The rank *lever* itself — that a
    // shock-aligned shell holds O(10) rank while capture grows — is gated at resolution in
    // `coordinate/rank_lever_3d_tests`.
    let (out, peak) = marcher.run(&state, 3).unwrap();
    assert!(
        out[0].iter().all(|d| d.is_finite() && *d > 0.0),
        "density stays positive & finite"
    );
    assert!(out[4].iter().all(|e| e.is_finite()), "energy stays finite");
    assert!(
        peak <= 1usize << ((3 * l) / 2),
        "peak bond within the full-rank ceiling: {peak}"
    );
}

/// Locate the radial front (steepest (i,j)-averaged density gradient along the z/ζ lattice).
fn front_k(density: &[f64], n: usize) -> usize {
    let mut prof = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..n {
            for (k, p) in prof.iter_mut().enumerate() {
                *p += density[(i * n + j) * n + k];
            }
        }
    }
    let (mut best, mut ks) = (-1.0f64, 0usize);
    for k in 2..n - 2 {
        let g = (prof[k + 1] - prof[k - 1]).abs();
        if g > best {
            best = g;
            ks = k;
        }
    }
    ks
}

#[test]
fn repin_engages_and_pins_the_radial_front_to_the_target_band() {
    // A radial density front at ζ-band k0 = 2, off the target band k = 5. The re-pin must roll it back to
    // the target (a rank-preserving relabel) and slide the shell; without re-pin the front stays put.
    // Bands are sized for the 8³ ζ grid (the front locator scans k ∈ 2..Nz-2 = {2,3,4,5}); the full 16³
    // sweep is the example/verification job.
    let l = 3usize;
    let n = 1usize << l;
    let tot = n * n * n;
    let (k0, target) = (2.0f64, 5usize);
    let (w, p) = (0.05, 1.0);
    let (mut rho, mut mx, mut my, mut mz, mut e) = (
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
    );
    for _i in 0..n {
        for _j in 0..n {
            for k in 0..n {
                let d = 1.2 - 0.4 * ((k as f64 - k0) / 1.5).tanh();
                rho.push(d);
                mx.push(0.0);
                my.push(0.0);
                mz.push(d * w);
                e.push(p / (GAMMA - 1.0) + 0.5 * d * w * w);
            }
        }
    }
    let state: EulerState3d<f64> = [rho, mx, my, mz, e];

    let shell =
        BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr()).unwrap();
    let dx = 1.0 / n as f64;
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, 0.0005, 2.0, tr()).unwrap();

    // Without re-pin the marcher barely moves the front — it stays near k0.
    let (out_static, _) = marcher.run(&state, 3).unwrap();
    let kf_static = front_k(&out_static[0], n);
    assert!(
        (kf_static as isize - k0 as isize).abs() <= 2,
        "without re-pin the front stays near k0: {kf_static}"
    );

    // With re-pin the front is relocated to the target band and the bond stays bounded.
    let (out, peak, n_repin) = marcher.run_repinned(&state, 3, target).unwrap();
    assert!(
        n_repin >= 1,
        "re-pin must engage when the front is off-band"
    );
    let kf = front_k(&out[0], n);
    assert!(
        (kf as isize - target as isize).abs() <= 1,
        "re-pin pins the front to the target band: {kf} vs {target}"
    );
    assert!(
        peak <= 1usize << ((3 * l) / 2),
        "re-pinned bond stays within the full-rank ceiling: {peak}"
    );
}

#[test]
fn new_rejects_non_positive_reference_speed() {
    // The fitted constructor rejects a non-finite / non-positive reference wave speed with a
    // `NumericalInstability`.
    let l = 3usize;
    let dx = 1.0 / (1usize << l) as f64;
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    match CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.001, 0.0, tr()) {
        Ok(_) => panic!("s_ref = 0 must error"),
        Err(e) => assert!(
            matches!(e.0, PhysicsErrorEnum::NumericalInstability(_)),
            "s_ref = 0 must be a NumericalInstability: {e:?}"
        ),
    }
    let ident2 = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    assert!(
        CompressibleMarcher3dFitted::new(ident2, dx, GAMMA, 0.001, -1.0, tr()).is_err(),
        "negative s_ref must also error"
    );
}

#[test]
fn metric_getter_returns_the_coordinate() {
    // The `metric()` accessor returns the coordinate the marcher runs over (here a body-fitted shell),
    // whose parameters must round-trip.
    let l = 3usize;
    let dx = 1.0 / (1usize << l) as f64;
    let shell =
        BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, 0.0005, 2.0, tr()).unwrap();
    let m = marcher.metric();
    assert!((m.r0() - 0.5).abs() < 1e-15, "metric r0 round-trips");
    assert!((m.dr() - 1.0).abs() < 1e-15, "metric dr round-trips");
}

#[test]
fn advance_matches_one_step_on_the_identity_chart() {
    // The `Marcher` trait impl for the fitted marcher: `advance` is exactly one `step`, and a uniform
    // state is a fixed point of it.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let q = |buf: Vec<f64>| {
        quantize_3d(
            &CausalTensor::new(buf, vec![1 << l, 1 << l, 1 << l]).unwrap(),
            &tr(),
        )
        .unwrap()
    };
    let (rho, u, v, w, p) = (1.1, 0.2, 0.1, 0.05, 0.7);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v + w * w);
    let state: EulerStateTt3d<f64> = [
        q(vec![rho; n]),
        q(vec![rho * u; n]),
        q(vec![rho * v; n]),
        q(vec![rho * w; n]),
        q(vec![e; n]),
    ];
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.001, 1.3, tr()).unwrap();

    let advanced = Marcher::advance(&marcher, &state, &()).unwrap();
    let stepped = marcher.step(&state).unwrap();
    let a0 = dequantize_3d(&advanced[0], l, l, l).unwrap();
    let s0 = dequantize_3d(&stepped[0], l, l, l).unwrap();
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
fn run_rejects_wrong_length_state() {
    // `run` guards each component buffer against a length ≠ 2^Lx·2^Ly·2^Lz.
    let l = 3usize;
    let dx = 1.0 / (1usize << l) as f64;
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.001, 1.3, tr()).unwrap();
    let bad: EulerState3d<f64> = [
        vec![1.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![2.5; 7],
    ];
    let err = marcher.run(&bad, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-length state must be a DimensionMismatch: {err:?}"
    );
}

#[test]
fn run_rejects_non_positive_density() {
    // The fitted flux/EOS enforces positivity: a negative-density cell must surface a
    // `PhysicalInvariantBroken`.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.001, 1.3, tr()).unwrap();
    let mut rho = vec![1.0; n];
    rho[11] = -0.2; // non-physical density
    let state: EulerState3d<f64> = [rho, vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    let err = marcher.run(&state, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "non-positive density must break the positivity invariant: {err:?}"
    );
}

#[test]
fn run_repinned_rejects_wrong_length_state() {
    // `run_repinned` (body-fitted shell only) guards the state length before marching.
    let l = 3usize;
    let dx = 1.0 / (1usize << l) as f64;
    let shell =
        BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, 0.0005, 2.0, tr()).unwrap();
    let bad: EulerState3d<f64> = [
        vec![1.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![2.5; 7],
    ];
    let err = marcher.run_repinned(&bad, 2, 4).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-length state must be a DimensionMismatch: {err:?}"
    );
}

#[test]
fn run_repinned_never_repins_on_a_thin_zeta_grid() {
    // With Nz < 5 the front locator returns `None` (too thin to bracket a front), so `run_repinned`
    // completes without ever re-pinning — exercising the thin-grid guard.
    let (lx, ly, lz) = (3usize, 3usize, 2usize); // Nz = 4 < 5
    let (nx, ny, nz) = (1usize << lx, 1usize << ly, 1usize << lz);
    let n = nx * ny * nz;
    let dx = 1.0 / nx as f64;
    let shell =
        BodyFittedCoordinate3d::<f64>::new(lx, ly, lz, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr()).unwrap();
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, 0.0005, 2.0, tr()).unwrap();
    let (p, w) = (1.0f64, 0.05f64);
    let e = p / (GAMMA - 1.0) + 0.5 * 1.0 * w * w;
    let state: EulerState3d<f64> = [
        vec![1.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![1.0 * w; n],
        vec![e; n],
    ];
    let (_out, _peak, n_repin) = marcher.run_repinned(&state, 3, 1).unwrap();
    assert_eq!(
        n_repin, 0,
        "a thin ζ grid (Nz < 5) can never locate a front, so no re-pin: {n_repin}"
    );
}

#[test]
fn non_positive_pressure_is_rejected() {
    // Item 12, fitted 3-D: the shared pressure guard refuses a non-hyperbolic cell here too.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let ident = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, tr()).unwrap();
    let fitted = CompressibleMarcher3dFitted::new(ident, dx, GAMMA, 0.002, 1.3, tr()).unwrap();
    let mut state: EulerState3d<f64> = [
        vec![1.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![2.5; n],
    ];
    state[1][n / 2] = 2.0;
    state[4][n / 2] = 1.0;
    let err = fitted.run(&state, 1).unwrap_err();
    assert!(
        matches!(
            err.0,
            deep_causality_physics::PhysicsErrorEnum::PhysicalInvariantBroken(_)
        ),
        "a non-hyperbolic cell must be refused in fitted 3-D: {err:?}"
    );
}
