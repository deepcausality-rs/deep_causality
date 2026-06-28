/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    QttProjector2d, divergence_residual, kinetic_energy, max_bond, max_speed, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, TensorTrain, Truncation};

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

fn tg_u(x: f64, y: f64) -> f64 {
    -(x.cos() * y.sin())
}
fn tg_v(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

#[test]
fn kinetic_energy_matches_dense() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let uf = field(dx, tg_u);
    let vf = field(dx, tg_v);
    let u = quantize_2d(&uf, &trunc).unwrap();
    let v = quantize_2d(&vf, &trunc).unwrap();

    // Dense reference: ½ Σ (u² + v²) over the grid coefficients (Frobenius/L2 energy).
    let dense: f64 = uf
        .as_slice()
        .iter()
        .zip(vf.as_slice())
        .map(|(a, b)| a * a + b * b)
        .sum::<f64>()
        * 0.5;

    let ke = kinetic_energy(&u, &v).unwrap();
    assert!((ke - dense).abs() <= 1e-9, "ke {ke} vs dense {dense}");
}

#[test]
fn divergence_residual_is_tt_native() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let u = quantize_2d(&field(dx, tg_u), &trunc).unwrap();
    let v = quantize_2d(&field(dx, tg_v), &trunc).unwrap();
    let projector = QttProjector2d::new(L, L, dx, dx, trunc).unwrap();

    // The Taylor–Green vortex is divergence-free, so the residual norm is ~ 0.
    let res = divergence_residual(&projector, &u, &v).unwrap();
    assert!(res <= 1e-9, "divergence residual {res} too large");

    // Agreement with the explicit divergence-train norm.
    let div_norm = projector.divergence(&u, &v).unwrap().norm().unwrap();
    assert!((res - div_norm).abs() <= 1e-12);
}

#[test]
fn max_bond_matches_cores() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let u = quantize_2d(&field(dx, tg_u), &trunc).unwrap();
    let v = quantize_2d(&field(dx, tg_v), &trunc).unwrap();

    let want = u
        .cores()
        .iter()
        .chain(v.cores().iter())
        .map(|c| c.shape()[2])
        .max()
        .unwrap();
    assert_eq!(max_bond(&u, &v), want);
}

#[test]
fn max_speed_matches_dense() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let uf = field(dx, tg_u);
    let vf = field(dx, tg_v);
    let u = quantize_2d(&uf, &trunc).unwrap();
    let v = quantize_2d(&vf, &trunc).unwrap();

    let dense = uf
        .as_slice()
        .iter()
        .zip(vf.as_slice())
        .map(|(a, b)| (a * a + b * b).sqrt())
        .fold(0.0f64, f64::max);

    let ms = max_speed(&u, &v, L, L).unwrap();
    assert!(
        (ms - dense).abs() <= 1e-9,
        "max_speed {ms} vs dense {dense}"
    );
}
