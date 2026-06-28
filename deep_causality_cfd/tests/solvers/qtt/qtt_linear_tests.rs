/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{Marcher, QttLinear1d, quantize};
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;

fn field(data: Vec<f64>) -> CausalTensor<f64> {
    let n = data.len();
    CausalTensor::new(data, vec![n]).unwrap()
}

#[test]
fn diffusion_matches_analytic() {
    // Pure diffusion of a single sine mode: u(x,t) = exp(−ν k² t)·sin(k x), periodic on [0, 1).
    let l = 6usize;
    let n = 1usize << l; // 64
    let lx = 1.0f64;
    let dx = lx / n as f64;
    let (c, nu) = (0.0f64, 0.01f64);
    let dt = 0.2 * dx * dx / (2.0 * nu); // comfortably within the explicit stability limit
    let k = TAU / lx;

    let u0: Vec<f64> = (0..n).map(|j| (k * j as f64 * dx).sin()).collect();
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let solver = QttLinear1d::new(l, dx, dt, c, nu, trunc).unwrap();

    let steps = 40usize;
    let got = solver.run(&field(u0), steps).unwrap();
    let t = dt * steps as f64;
    let decay = (-nu * k * k * t).exp();
    for (j, &g) in got.as_slice().iter().enumerate() {
        let want = decay * (k * j as f64 * dx).sin();
        assert!((g - want).abs() <= 3e-3, "j={j}: {g} vs {want}");
    }
}

#[test]
fn bounded_rank_under_recompression() {
    // A sine stays QTT-rank ~2 under diffusion; recompression must keep the bond bounded over many steps.
    let l = 7usize;
    let n = 1usize << l;
    let dx = 1.0f64 / n as f64;
    let nu = 0.02f64;
    let dt = 0.2 * dx * dx / (2.0 * nu);
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();
    let solver = QttLinear1d::new(l, dx, dt, 0.0, nu, trunc).unwrap();

    let u0: Vec<f64> = (0..n).map(|j| (TAU * j as f64 / n as f64).sin()).collect();
    let mut state = quantize(&field(u0), &trunc).unwrap();
    for _ in 0..300 {
        state = solver.advance(&state, &()).unwrap();
        let bond = state
            .cores()
            .iter()
            .map(|core| core.shape()[2])
            .max()
            .unwrap();
        assert!(bond <= 8, "bond grew under recompression: {bond}");
    }
}

#[test]
fn pure_advection_preserves_the_mean() {
    // ν = 0: centered advection conserves the discrete mean (mode 0). Use an exact round so the
    // recompression does not perturb it.
    let l = 6usize;
    let n = 1usize << l;
    let dx = 1.0f64 / n as f64;
    let c = 0.5f64;
    let dt = 0.3 * dx / c;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let solver = QttLinear1d::new(l, dx, dt, c, 0.0, trunc).unwrap();

    let u0: Vec<f64> = (0..n)
        .map(|j| 1.5 + (TAU * j as f64 / n as f64).cos())
        .collect();
    let mean0 = u0.iter().sum::<f64>() / n as f64;
    let got = solver.run(&field(u0), 25).unwrap();
    let mean1 = got.as_slice().iter().sum::<f64>() / n as f64;
    assert!(
        (mean1 - mean0).abs() <= 1e-9,
        "mean drifted: {mean0} -> {mean1}"
    );
}
