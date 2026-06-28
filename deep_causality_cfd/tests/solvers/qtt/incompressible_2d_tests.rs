/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{Marcher, QttIncompressible2d, dequantize_2d, quantize_2d};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

fn field(dx: f64, f: impl Fn(f64, f64) -> f64) -> CausalTensor<f64> {
    let mut data = vec![0.0; N * N];
    for i in 0..N {
        for j in 0..N {
            data[i * N + j] = f(i as f64 * dx, j as f64 * dx);
        }
    }
    CausalTensor::new(data, vec![N, N]).unwrap()
}

// The 2-D Taylor–Green vortex: u = −cos(x)sin(y), v = sin(x)cos(y); the velocity decays as e^{−2νt}
// (the nonlinear convection is a pure gradient, removed by the projection).
fn tg_u(x: f64, y: f64) -> f64 {
    -(x.cos() * y.sin())
}
fn tg_v(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

#[test]
fn taylor_green_vortex_decays() {
    let dx = TAU / N as f64;
    let (nu, dt, steps) = (0.05f64, 0.02f64, 10usize);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let (u, v) = solver
        .run(&field(dx, tg_u), &field(dx, tg_v), steps)
        .unwrap();
    let decay = (-2.0 * nu * dt * steps as f64).exp();
    let (us, vs) = (u.as_slice(), v.as_slice());
    let mut max_err = 0.0f64;
    for i in 0..N {
        for j in 0..N {
            let (x, y) = (i as f64 * dx, j as f64 * dx);
            max_err = max_err
                .max((us[i * N + j] - tg_u(x, y) * decay).abs())
                .max((vs[i * N + j] - tg_v(x, y) * decay).abs());
        }
    }
    assert!(max_err <= 2e-2, "Taylor–Green decay error {max_err}");
}

fn max_divergence(u: &CausalTensor<f64>, v: &CausalTensor<f64>, dx: f64) -> f64 {
    let (us, vs) = (u.as_slice(), v.as_slice());
    let mut m = 0.0f64;
    for i in 0..N {
        for j in 0..N {
            let dudx = (us[((i + 1) % N) * N + j] - us[((i + N - 1) % N) * N + j]) / (2.0 * dx);
            let dvdy = (vs[i * N + (j + 1) % N] - vs[i * N + (j + N - 1) % N]) / (2.0 * dx);
            m = m.max((dudx + dvdy).abs());
        }
    }
    m
}

#[test]
fn stays_divergence_free_and_bounded_rank() {
    let dx = TAU / N as f64;
    let (nu, dt) = (0.05f64, 0.02f64);
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let mut state: (CausalTensorTrain<f64>, CausalTensorTrain<f64>) = (
        quantize_2d(&field(dx, tg_u), &trunc).unwrap(),
        quantize_2d(&field(dx, tg_v), &trunc).unwrap(),
    );
    for _ in 0..30 {
        state = solver.advance(&state, &()).unwrap();
        let bond = state.0.cores().iter().map(|c| c.shape()[2]).max().unwrap();
        assert!(bond <= 12, "bond grew under recompression: {bond}");
        let u = dequantize_2d(&state.0, L, L).unwrap();
        let v = dequantize_2d(&state.1, L, L).unwrap();
        let div = max_divergence(&u, &v, dx);
        assert!(div <= 1e-5, "divergence grew: {div}");
    }
}
