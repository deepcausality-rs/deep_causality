/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{dequantize, gradient, laplacian, quantize, shift_minus, shift_plus};
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
