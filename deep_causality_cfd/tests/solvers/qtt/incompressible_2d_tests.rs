/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{Marcher, QttIncompressible2d, dequantize_2d, quantize_2d};
use deep_causality_physics::PhysicsErrorEnum;
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
fn scalar_rate_is_zero_for_a_uniform_field_at_rest() {
    // rate = −(u·∇)s + κ·∇²s. With u = v = 0 (no advection) and s constant (∇²s = 0), the rate must
    // vanish identically — exercising the passive-scalar transport seam the immersed marcher rides.
    let dx = TAU / N as f64;
    let (nu, dt) = (0.05f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let zero = quantize_2d(&field(dx, |_x, _y| 0.0), &trunc).unwrap();
    let s = quantize_2d(&field(dx, |_x, _y| 3.3), &trunc).unwrap();
    let rate = solver.scalar_rate(&s, &zero, &zero, 0.02).unwrap();
    let rd = dequantize_2d(&rate, L, L).unwrap();
    for v in rd.as_slice() {
        assert!(
            v.abs() < 1e-9,
            "scalar rate of a uniform field at rest must vanish: {v}"
        );
    }
}

#[test]
fn scalar_rate_diffuses_a_sinusoid() {
    // With u = v = 0 the rate reduces to pure diffusion κ·∇²s. For s = sin(x) the Laplacian is −sin(x),
    // so κ·∇²s ≈ −κ·s (up to O(dx²)): the rate has the opposite sign of s.
    let dx = TAU / N as f64;
    let (nu, dt, kappa) = (0.05f64, 0.02f64, 0.1f64);
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let zero = quantize_2d(&field(dx, |_x, _y| 0.0), &trunc).unwrap();
    let s = quantize_2d(&field(dx, |x, _y| x.sin()), &trunc).unwrap();
    let rate = solver.scalar_rate(&s, &zero, &zero, kappa).unwrap();
    let rd = dequantize_2d(&rate, L, L).unwrap();
    let sd = dequantize_2d(&s, L, L).unwrap();
    let mut max_err = 0.0f64;
    for (r, sv) in rd.as_slice().iter().zip(sd.as_slice()) {
        max_err = max_err.max((r - (-kappa * sv)).abs());
    }
    assert!(max_err < 5e-3, "diffusion rate should be ≈ −κ·s: {max_err}");
}

#[test]
fn rate_pair_convection_matches_the_analytic_reference_with_nonzero_velocity() {
    // The shipped convection path `rate_pair` — the one the immersed-body marcher rides — carries
    // `(−(u·∇)u + ν∇²u, −(u·∇)v + ν∇²v)` with no projection, so a nonzero-velocity convection term
    // survives here. The full-solver Taylor–Green test cannot check it: there the convection is a
    // pure gradient the projection annihilates, so a sign flip in the shipped convection is invisible.
    //
    // Independent reference: u = sin(x), v = sin(y) on [0, 2π]². Then
    //   (u·∇)u = u·∂ₓu = ½sin(2x),  ∇²u = −sin(x)  ⇒  ru = −½sin(2x) − ν·sin(x);
    //   (u·∇)v = v·∂ᵧv = ½sin(2y),  ∇²v = −sin(y)  ⇒  rv = −½sin(2y) − ν·sin(y).
    // The bound covers the centered-FD truncation at 64² (dx ≈ 0.098; the sin(2·) term carries the
    // largest truncation, ≈ 0.2 % of its 0.5 amplitude). A sign flip in the shipped convection changes
    // each rate by up to 1.0, far outside the bound, so this test bites on one.
    const LC: usize = 6;
    const NC: usize = 1 << LC;
    let dx = TAU / NC as f64;
    let nu = 0.05f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let solver = QttIncompressible2d::new(LC, LC, dx, dx, 0.001, nu, trunc).unwrap();

    let mk = |f: fn(f64, f64) -> f64| {
        let mut data = vec![0.0f64; NC * NC];
        for i in 0..NC {
            for j in 0..NC {
                data[i * NC + j] = f(i as f64 * dx, j as f64 * dx);
            }
        }
        quantize_2d(&CausalTensor::new(data, vec![NC, NC]).unwrap(), &trunc).unwrap()
    };
    let u = mk(|x, _y| x.sin());
    let v = mk(|_x, y| y.sin());

    let (ru, rv) = solver.rate_pair(&u, &v).unwrap();
    let rud = dequantize_2d(&ru, LC, LC).unwrap();
    let rvd = dequantize_2d(&rv, LC, LC).unwrap();

    let mut max_err = 0.0f64;
    for i in 0..NC {
        for j in 0..NC {
            let (x, y) = (i as f64 * dx, j as f64 * dx);
            let ru_ref = -0.5 * (2.0 * x).sin() - nu * x.sin();
            let rv_ref = -0.5 * (2.0 * y).sin() - nu * y.sin();
            max_err = max_err
                .max((rud.as_slice()[i * NC + j] - ru_ref).abs())
                .max((rvd.as_slice()[i * NC + j] - rv_ref).abs());
        }
    }
    assert!(
        max_err < 3e-3,
        "rate_pair convection+diffusion vs the analytic reference: {max_err}"
    );
}

#[test]
fn run_rejects_wrong_shape_fields() {
    let dx = TAU / N as f64;
    let (nu, dt) = (0.05f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let solver = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let bad = CausalTensor::new(vec![0.0f64; N * (N / 2)], vec![N, N / 2]).unwrap();
    let good = field(dx, |_x, _y| 0.0);
    let err = solver.run(&bad, &good, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-shape field must be a DimensionMismatch: {err:?}"
    );
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

// --- Numerical-envelope validation (close-qtt-solver-envelope, item 13) ---------------------------

#[test]
fn a_negative_viscosity_is_refused() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    assert!(
        QttIncompressible2d::new(L, L, dx, dx, 0.005, -0.1, trunc).is_err(),
        "a negative kinematic viscosity must be refused"
    );
}

#[test]
fn a_non_positive_spacing_is_refused() {
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    assert!(
        QttIncompressible2d::new(L, L, 0.0, 0.1, 0.005, 0.05, trunc).is_err(),
        "dx = 0 must be refused"
    );
}

#[test]
fn a_dt_beyond_the_diffusive_limit_is_refused_and_names_it() {
    let dx = TAU / N as f64; // 0.3927; dx²/(4·0.05) = 0.771
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let Err(err) = QttIncompressible2d::new(L, L, dx, dx, 1.0, 0.05, trunc) else {
        panic!("dt = 1.0 exceeds the diffusive limit 0.771");
    };
    let msg = format!("{err:?}");
    assert!(
        msg.contains("diffusive") && msg.contains('1'),
        "the diagnostic must name the diffusive limit and the values: {msg}"
    );
}
