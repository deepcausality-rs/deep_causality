/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    Marcher, QttImmersed2d, QttIncompressible2d, body_mask_2d, dequantize_2d, quantize_2d,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

fn uniform(value: f64) -> CausalTensor<f64> {
    CausalTensor::new(vec![value; N * N], vec![N, N]).unwrap()
}

fn zeros() -> CausalTensor<f64> {
    uniform(0.0)
}

// A centered cylinder mask covering the middle of the box.
fn cyl_mask(dx: f64, trunc: &Truncation<f64>) -> CausalTensorTrain<f64> {
    let c = TAU * 0.5;
    body_mask_2d::<f64>(L, L, dx, dx, c, c, TAU * 0.18, 2.0 * dx, trunc).unwrap()
}

#[test]
fn no_slip_drives_interior_velocity_to_zero() {
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64); // dt/eta = 0.25 (explicit-stable)
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver =
        QttImmersed2d::new(L, L, dx, dx, dt, nu, mask.clone(), 0.0, 0.0, eta, trunc).unwrap();

    // Seed a uniform free-stream u = 1, v = 0 (rank-1); the body should brake the flow inside it.
    let (u, _v) = solver.run(&uniform(1.0), &zeros(), 40).unwrap();
    let md = dequantize_2d(&mask, L, L).unwrap();
    let (us, ms) = (u.as_slice(), md.as_slice());

    // Mean speed inside the body (mask > 0.9) vs. outside (mask < 0.1).
    let (mut in_sum, mut in_n, mut out_sum, mut out_n) = (0.0, 0, 0.0, 0);
    for k in 0..N * N {
        if ms[k] > 0.9 {
            in_sum += us[k].abs();
            in_n += 1;
        } else if ms[k] < 0.1 {
            out_sum += us[k].abs();
            out_n += 1;
        }
    }
    let inside = in_sum / in_n as f64;
    let outside = out_sum / out_n as f64;
    assert!(
        in_n > 0 && out_n > 0,
        "mask did not separate interior/exterior"
    );
    assert!(
        inside < 0.2,
        "no-slip not enforced: interior mean speed {inside}"
    );
    assert!(
        outside > 0.6,
        "free-stream collapsed outside the body: {outside}"
    );
}

#[test]
fn stays_divergence_free_and_bounded_rank() {
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64);
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver = QttImmersed2d::new(L, L, dx, dx, dt, nu, mask, 0.0, 0.0, eta, trunc).unwrap();

    let mut state: (CausalTensorTrain<f64>, CausalTensorTrain<f64>) = (
        quantize_2d(&uniform(1.0), &trunc).unwrap(),
        quantize_2d(&zeros(), &trunc).unwrap(),
    );
    for _ in 0..30 {
        state = solver.advance(&state, &()).unwrap();
        let bond = state.0.cores().iter().map(|c| c.shape()[2]).max().unwrap();
        assert!(bond <= 24, "bond grew under recompression: {bond}");
        let u = dequantize_2d(&state.0, L, L).unwrap();
        let v = dequantize_2d(&state.1, L, L).unwrap();
        let div = max_divergence(&u, &v, dx);
        assert!(div <= 1e-5, "divergence grew: {div}");
    }
}

#[test]
fn zero_mask_reduces_to_the_body_free_solver() {
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.01f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();

    // A genuinely empty body (mask ≡ 0): penalization vanishes.
    let zero_mask = quantize_2d(&zeros(), &trunc).unwrap();
    let immersed =
        QttImmersed2d::new(L, L, dx, dx, dt, nu, zero_mask, 0.0, 0.0, eta, trunc).unwrap();
    let plain = QttIncompressible2d::new(L, L, dx, dx, dt, nu, trunc).unwrap();

    let u0 = field(dx, |x, y| -(x.cos() * y.sin()));
    let v0 = field(dx, |x, y| x.sin() * y.cos());
    let (iu, iv) = immersed.run(&u0, &v0, 8).unwrap();
    let (pu, pv) = plain.run(&u0, &v0, 8).unwrap();

    for (a, b) in iu.as_slice().iter().zip(pu.as_slice()) {
        assert!(
            (a - b).abs() <= 1e-10,
            "u diverged from body-free: {a} vs {b}"
        );
    }
    for (a, b) in iv.as_slice().iter().zip(pv.as_slice()) {
        assert!(
            (a - b).abs() <= 1e-10,
            "v diverged from body-free: {a} vs {b}"
        );
    }
}

#[test]
fn accessors_report_construction_parameters() {
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver =
        QttImmersed2d::new(L, L, dx, dx, dt, nu, mask.clone(), 0.7, -0.3, eta, trunc).unwrap();

    assert_eq!(solver.modes(), (L, L), "modes");
    assert!((solver.eta() - eta).abs() < 1e-15, "eta getter");
    let (ubx, uby) = solver.body_velocity();
    assert!(
        (ubx - 0.7).abs() < 1e-15 && (uby + 0.3).abs() < 1e-15,
        "body velocity"
    );
    // The mask getter returns the same train the projector observables read.
    let got = dequantize_2d(solver.mask(), L, L).unwrap();
    let want = dequantize_2d(&mask, L, L).unwrap();
    for (a, b) in got.as_slice().iter().zip(want.as_slice()) {
        assert!((a - b).abs() < 1e-12, "mask getter mismatch: {a} vs {b}");
    }
    // The projector delegates to the base solver (present, usable).
    let _ = solver.projector();
}

#[test]
fn moving_body_penalization_drives_interior_to_body_velocity() {
    // A moving wall (u_body = 1) exercises the `ub != 0` deficit branch of `penalize`: the interior
    // velocity is driven toward the body velocity, not toward zero.
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver =
        QttImmersed2d::new(L, L, dx, dx, dt, nu, mask.clone(), 1.0, 0.0, eta, trunc).unwrap();

    // Seed a quiescent field u = 0; the moving body should pull the interior toward u_body = 1.
    let (u, _v) = solver.run(&zeros(), &zeros(), 40).unwrap();
    let md = dequantize_2d(&mask, L, L).unwrap();
    let (us, ms) = (u.as_slice(), md.as_slice());
    let (mut in_sum, mut in_n) = (0.0, 0);
    for k in 0..N * N {
        if ms[k] > 0.9 {
            in_sum += us[k];
            in_n += 1;
        }
    }
    assert!(in_n > 0, "mask has no solid interior");
    let inside = in_sum / in_n as f64;
    assert!(
        inside > 0.6,
        "moving body did not drag the interior toward u_body = 1: {inside}"
    );
}

#[test]
fn advance_scalar_penalizes_temperature_to_the_wall() {
    // A passive scalar advected on a quiescent rollout: inside the body the temperature relaxes toward
    // the wall temperature t_wall; there is no projection (a scalar has no incompressibility constraint).
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver =
        QttImmersed2d::new(L, L, dx, dx, dt, nu, mask.clone(), 0.0, 0.0, eta, trunc).unwrap();

    // Quiescent velocity (u = v = 0) and a cold uniform field; wall is hot (t_wall = 1).
    let u = quantize_2d(&zeros(), &trunc).unwrap();
    let v = quantize_2d(&zeros(), &trunc).unwrap();
    let mut temp = quantize_2d(&zeros(), &trunc).unwrap();
    for _ in 0..40 {
        temp = solver.advance_scalar(&temp, &u, &v, 1.0, 0.01).unwrap();
    }
    let td = dequantize_2d(&temp, L, L).unwrap();
    let md = dequantize_2d(&mask, L, L).unwrap();
    let (ts, ms) = (td.as_slice(), md.as_slice());
    let (mut in_sum, mut in_n, mut out_sum, mut out_n) = (0.0, 0, 0.0, 0);
    for k in 0..N * N {
        if ms[k] > 0.9 {
            in_sum += ts[k];
            in_n += 1;
        } else if ms[k] < 0.1 {
            out_sum += ts[k];
            out_n += 1;
        }
    }
    assert!(
        in_n > 0 && out_n > 0,
        "mask did not separate interior/exterior"
    );
    let inside = in_sum / in_n as f64;
    let outside = out_sum / out_n as f64;
    // The wall (t_wall = 1) heats the solid interior toward 1; the far field stays cooler than the wall.
    assert!(
        inside > 0.6,
        "wall did not heat the interior scalar: {inside}"
    );
    assert!(
        inside > outside,
        "interior should be hotter than the exterior: {inside} vs {outside}"
    );
}

#[test]
fn run_rejects_wrong_shape_fields() {
    let dx = TAU / N as f64;
    let (nu, dt, eta) = (0.05f64, 0.005f64, 0.02f64);
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    let mask = cyl_mask(dx, &trunc);
    let solver = QttImmersed2d::new(L, L, dx, dx, dt, nu, mask, 0.0, 0.0, eta, trunc).unwrap();

    // A field of the wrong shape ([N, N/2] instead of [N, N]) must surface a DimensionMismatch.
    let bad = CausalTensor::new(vec![0.0f64; N * (N / 2)], vec![N, N / 2]).unwrap();
    let good = zeros();
    let err = solver.run(&bad, &good, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)),
        "wrong-shape field must be a DimensionMismatch: {err:?}"
    );
}

fn field(dx: f64, f: impl Fn(f64, f64) -> f64) -> CausalTensor<f64> {
    let mut data = vec![0.0; N * N];
    for i in 0..N {
        for j in 0..N {
            data[i * N + j] = f(i as f64 * dx, j as f64 * dx);
        }
    }
    CausalTensor::new(data, vec![N, N]).unwrap()
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

// --- Numerical-envelope validation (close-qtt-solver-envelope, item 13) ---------------------------
//
// The QTT constructors previously validated nothing — they destructured straight into Ok(Self{..}),
// so η = 0 gave a −∞ forcing and marched. They now refuse an out-of-envelope configuration with the
// same "name the limit and both values" quality the DEC family's cfl_check has.

#[test]
fn a_non_positive_eta_is_refused() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let mask = cyl_mask(dx, &trunc);
    // η = 0 (the −1/η = −inf case) and η < 0 both refused.
    assert!(
        QttImmersed2d::new(
            L,
            L,
            dx,
            dx,
            0.005,
            0.05,
            mask.clone(),
            0.0,
            0.0,
            0.0,
            trunc
        )
        .is_err(),
        "η = 0 must be refused (the forcing would be infinite)"
    );
    assert!(
        QttImmersed2d::new(L, L, dx, dx, 0.005, 0.05, mask, 0.0, 0.0, -0.01, trunc).is_err(),
        "η < 0 must be refused"
    );
}

#[test]
fn a_dt_beyond_the_penalization_limit_is_refused_and_names_it() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let mask = cyl_mask(dx, &trunc);
    // η = 0.02 ⇒ 2η = 0.04. dt = 0.05 > 0.04 trips the penalization limit; 0.05 < dx²/(4ν) = 0.771,
    // so the diffusive limit is not the one firing.
    let (dt, eta) = (0.05f64, 0.02f64);
    let Err(err) = QttImmersed2d::new(L, L, dx, dx, dt, 0.05, mask, 0.0, 0.0, eta, trunc) else {
        panic!("dt beyond 2η must be refused");
    };
    let msg = format!("{err:?}");
    assert!(
        msg.contains("penalization") && msg.contains("0.05") && msg.contains("0.04"),
        "the diagnostic must name the limit and both values: {msg}"
    );
}

#[test]
fn an_in_envelope_configuration_constructs() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let mask = cyl_mask(dx, &trunc);
    // dt/η = 0.25 (well inside 2η), dt < dx²/(4ν): the shipped-style configuration.
    assert!(
        QttImmersed2d::new(L, L, dx, dx, 0.005, 0.05, mask, 0.0, 0.0, 0.02, trunc).is_ok(),
        "an in-envelope configuration must construct"
    );
}
