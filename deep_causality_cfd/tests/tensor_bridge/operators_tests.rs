/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{dequantize, gradient, laplacian, quantize, shift_minus, shift_plus};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

const TOL: f64 = 1e-10;

fn field(data: Vec<f64>) -> CausalTensor<f64> {
    let n = data.len();
    CausalTensor::new(data, vec![n]).unwrap()
}

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

/// Dequantize(op · quantize(u)) — the operator's real-space action on a field.
fn apply_to_field(op: &CausalTensorTrainOperator<f64>, u: &CausalTensor<f64>) -> Vec<f64> {
    let q = quantize(u, &full()).unwrap();
    let out = op.apply(&q, &full()).unwrap();
    dequantize(&out).unwrap().as_slice().to_vec()
}

#[test]
fn shift_plus_is_cyclic_forward() {
    // (S₊·u)[k] = u[(k−1) mod N].
    let u = field(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]); // N = 8, L = 3
    let got = apply_to_field(&shift_plus::<f64>(3).unwrap(), &u);
    for (k, &g) in got.iter().enumerate() {
        let want = u.as_slice()[(k + 8 - 1) % 8];
        assert!((g - want).abs() <= TOL, "k={k}: {g} vs {want}");
    }
}

#[test]
fn shift_minus_inverts_shift_plus() {
    let u = field((0..8).map(|i| (i as f64).sin()).collect());
    let (sp, sm) = (
        shift_plus::<f64>(3).unwrap(),
        shift_minus::<f64>(3).unwrap(),
    );
    let q = quantize(&u, &full()).unwrap();
    let back = sm.apply(&sp.apply(&q, &full()).unwrap(), &full()).unwrap();
    for (a, b) in dequantize(&back)
        .unwrap()
        .as_slice()
        .iter()
        .zip(u.as_slice())
    {
        assert!((a - b).abs() <= TOL, "S₋∘S₊ ≠ I: {a} vs {b}");
    }
}

#[test]
fn laplacian_matches_dense_stencil() {
    let (n, l, dx) = (16usize, 4usize, 0.5f64);
    let u: Vec<f64> = (0..n)
        .map(|i| (core::f64::consts::TAU * i as f64 / n as f64).sin())
        .collect();
    let got = apply_to_field(
        &laplacian::<f64>(l, dx, &full()).unwrap(),
        &field(u.clone()),
    );
    for (k, &g) in got.iter().enumerate() {
        let want = (u[(k + 1) % n] + u[(k + n - 1) % n] - 2.0 * u[k]) / (dx * dx);
        assert!((g - want).abs() <= 1e-9, "k={k}: {g} vs {want}");
    }
}

#[test]
fn shift_plus_rejects_zero_modes() {
    // l == 0 is the degenerate guard: a shift needs at least one mode.
    let err = shift_plus::<f64>(0).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
    // shift_minus is the transpose of shift_plus, so it propagates the same guard.
    assert!(shift_minus::<f64>(0).is_err());
}

#[test]
fn shift_plus_single_mode_is_the_2_cycle_swap() {
    // l == 1 is a distinct branch: the sole mode is both MSB and LSB, so S₊ = NOT (swap [a, b]).
    let u = field(vec![3.0, 7.0]); // N = 2, L = 1
    let got = apply_to_field(&shift_plus::<f64>(1).unwrap(), &u);
    // (S₊·u)[k] = u[(k−1) mod 2] ⇒ [u[1], u[0]] = [7, 3].
    assert!((got[0] - 7.0).abs() <= TOL, "got {got:?}");
    assert!((got[1] - 3.0).abs() <= TOL, "got {got:?}");
    // shift_minus(1) inverts it back to the original.
    let sm = shift_minus::<f64>(1).unwrap();
    let sp = shift_plus::<f64>(1).unwrap();
    let q = quantize(&u, &full()).unwrap();
    let back = dequantize(&sm.apply(&sp.apply(&q, &full()).unwrap(), &full()).unwrap()).unwrap();
    for (a, b) in back.as_slice().iter().zip(u.as_slice()) {
        assert!((a - b).abs() <= TOL, "S₋∘S₊ ≠ I at L=1: {a} vs {b}");
    }
}

#[test]
fn gradient_and_laplacian_reject_zero_modes() {
    // gradient/laplacian build shifts internally, so l == 0 propagates the shift guard.
    assert!(gradient::<f64>(0, 0.5, &full()).is_err());
    assert!(laplacian::<f64>(0, 0.5, &full()).is_err());
}

#[test]
fn gradient_matches_dense_and_annihilates_constant() {
    let (n, l, dx) = (16usize, 4usize, 0.25f64);
    let grad = gradient::<f64>(l, dx, &full()).unwrap();

    // A constant field differentiates to ~zero.
    for g in apply_to_field(&grad, &field(vec![3.7; n])) {
        assert!(g.abs() <= 1e-9, "constant gradient not zero: {g}");
    }

    // A smooth profile matches the centered difference.
    let u: Vec<f64> = (0..n)
        .map(|i| (core::f64::consts::TAU * i as f64 / n as f64).cos())
        .collect();
    let got = apply_to_field(&grad, &field(u.clone()));
    for (k, &g) in got.iter().enumerate() {
        let want = (u[(k + 1) % n] - u[(k + n - 1) % n]) / (2.0 * dx);
        assert!((g - want).abs() <= 1e-9, "k={k}: {g} vs {want}");
    }
}
