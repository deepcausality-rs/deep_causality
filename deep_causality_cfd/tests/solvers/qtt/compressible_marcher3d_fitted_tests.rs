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
    CompressibleMarcher3dFitted, EulerState3d,
};
use deep_causality_tensor::Truncation;

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
