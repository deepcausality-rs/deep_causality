/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{QttProjector2d, dequantize_2d, quantize_2d};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

fn vel(f: impl Fn(f64, f64) -> f64) -> CausalTensorTrain<f64> {
    let dx = TAU / N as f64;
    let mut data = vec![0.0; N * N];
    for i in 0..N {
        for j in 0..N {
            data[i * N + j] = f(i as f64 * dx, j as f64 * dx);
        }
    }
    quantize_2d(&CausalTensor::new(data, vec![N, N]).unwrap(), &full()).unwrap()
}

fn proj() -> QttProjector2d<f64> {
    let dx = TAU / N as f64;
    QttProjector2d::new(L, L, dx, dx, full()).unwrap()
}

fn max_abs_div(
    p: &QttProjector2d<f64>,
    u: &CausalTensorTrain<f64>,
    v: &CausalTensorTrain<f64>,
) -> f64 {
    let d = p.divergence(u, v).unwrap();
    dequantize_2d(&d, L, L)
        .unwrap()
        .as_slice()
        .iter()
        .map(|x| x.abs())
        .fold(0.0, f64::max)
}

#[test]
fn projection_removes_divergence() {
    let p = proj();
    // An arbitrary (non-solenoidal) smooth field, frequencies well below Nyquist.
    let u = vel(|x, y| x.sin() * (2.0 * y).cos());
    let v = vel(|x, y| (3.0 * x).cos() * y.sin());
    let div0 = max_abs_div(&p, &u, &v);
    let (un, vn) = p.project(&u, &v).unwrap();
    let div1 = max_abs_div(&p, &un, &vn);
    assert!(div1 <= 1e-8, "divergence not removed: {div0} -> {div1}");
    assert!(div1 < div0, "projection did nothing: {div0} -> {div1}");
}

#[test]
fn projection_is_idempotent() {
    let p = proj();
    let u = vel(|x, y| x.sin() + y.cos());
    let v = vel(|x, _y| (2.0 * x).cos());
    let (u1, v1) = p.project(&u, &v).unwrap();
    let (u2, v2) = p.project(&u1, &v1).unwrap();
    let cmp = |a: &CausalTensorTrain<f64>, b: &CausalTensorTrain<f64>| {
        let (da, db) = (
            dequantize_2d(a, L, L).unwrap(),
            dequantize_2d(b, L, L).unwrap(),
        );
        for (x, y) in da.as_slice().iter().zip(db.as_slice()) {
            assert!(
                (x - y).abs() <= 1e-8,
                "projection not idempotent: {x} vs {y}"
            );
        }
    };
    cmp(&u1, &u2);
    cmp(&v1, &v2);
}

#[test]
fn projected_field_is_finite() {
    let p = proj();
    let u = vel(|x, y| (x + y).sin());
    let v = vel(|x, y| (x - y).cos());
    let (un, vn) = p.project(&u, &v).unwrap();
    for x in dequantize_2d(&un, L, L).unwrap().as_slice() {
        assert!(x.is_finite(), "non-finite u");
    }
    for x in dequantize_2d(&vn, L, L).unwrap().as_slice() {
        assert!(x.is_finite(), "non-finite v");
    }
}
