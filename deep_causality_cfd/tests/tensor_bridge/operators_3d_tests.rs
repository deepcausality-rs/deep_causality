/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 0: the 3-D QTT codec and finite-difference operators — round-trip, axis isolation,
//! analytic-derivative agreement, divergence, bounded rank, and dimension-mismatch handling.

use deep_causality_cfd::{
    dequantize_3d, divergence_3d, gradient_x_3d, gradient_y_3d, gradient_z_3d, laplacian_3d,
    quantize_3d,
};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

const TAU: f64 = core::f64::consts::TAU;

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

fn field3d(
    nx: usize,
    ny: usize,
    nz: usize,
    f: impl Fn(usize, usize, usize) -> f64,
) -> CausalTensor<f64> {
    let mut data = vec![0.0; nx * ny * nz];
    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                data[(i * ny + j) * nz + k] = f(i, j, k);
            }
        }
    }
    CausalTensor::new(data, vec![nx, ny, nz]).unwrap()
}

fn apply_3d(op: &CausalTensorTrainOperator<f64>, u: &CausalTensor<f64>, l: usize) -> Vec<f64> {
    let q = quantize_3d(u, &full()).unwrap();
    let out = op.apply(&q, &full()).unwrap();
    dequantize_3d(&out, l, l, l).unwrap().as_slice().to_vec()
}

#[test]
fn round_trips_3d() {
    let (nx, ny, nz) = (4usize, 4usize, 4usize);
    let u = field3d(nx, ny, nz, |i, j, k| {
        (0.3 * (i * ny * nz + j * nz + k) as f64).sin() - 0.2 * k as f64
    });
    let q = quantize_3d(&u, &full()).unwrap();
    let back = dequantize_3d(&q, 2, 2, 2).unwrap();
    assert_eq!(back.shape(), &[4, 4, 4]);
    for (a, b) in back.as_slice().iter().zip(u.as_slice()) {
        assert!((a - b).abs() <= 1e-12, "3-D round-trip differs: {a} vs {b}");
    }
}

#[test]
fn gradient_x_hits_only_x() {
    let (n, dx, l) = (4usize, 0.5f64, 2usize);
    let s = |i: usize| (TAU * i as f64 / n as f64).sin();
    let u = field3d(n, n, n, |i, _j, _k| s(i));
    let gx = apply_3d(&gradient_x_3d::<f64>(l, l, l, dx, &full()).unwrap(), &u, l);
    let gy = apply_3d(&gradient_y_3d::<f64>(l, l, l, dx, &full()).unwrap(), &u, l);
    let gz = apply_3d(&gradient_z_3d::<f64>(l, l, l, dx, &full()).unwrap(), &u, l);
    for i in 0..n {
        let want = (s((i + 1) % n) - s((i + n - 1) % n)) / (2.0 * dx);
        for j in 0..n {
            for k in 0..n {
                let idx = (i * n + j) * n + k;
                assert!((gx[idx] - want).abs() <= 1e-9, "∂ₓ wrong at {i},{j},{k}");
                assert!(gy[idx].abs() <= 1e-9, "∂ᵧ of x-only field nonzero");
                assert!(gz[idx].abs() <= 1e-9, "∂_z of x-only field nonzero");
            }
        }
    }
}

#[test]
fn gradient_y_and_z_hit_their_axes() {
    let (n, d, l) = (4usize, 0.5f64, 2usize);
    let s = |m: usize| (TAU * m as f64 / n as f64).sin();
    // y-only field.
    let uy = field3d(n, n, n, |_i, j, _k| s(j));
    let gy = apply_3d(&gradient_y_3d::<f64>(l, l, l, d, &full()).unwrap(), &uy, l);
    // z-only field.
    let uz = field3d(n, n, n, |_i, _j, k| s(k));
    let gz = apply_3d(&gradient_z_3d::<f64>(l, l, l, d, &full()).unwrap(), &uz, l);
    for m in 0..n {
        let want = (s((m + 1) % n) - s((m + n - 1) % n)) / (2.0 * d);
        for a in 0..n {
            for b in 0..n {
                assert!((gy[(a * n + m) * n + b] - want).abs() <= 1e-9, "∂ᵧ wrong");
                assert!((gz[(a * n + b) * n + m] - want).abs() <= 1e-9, "∂_z wrong");
            }
        }
    }
}

#[test]
fn laplacian_3d_matches_seven_point() {
    let (n, d, l) = (4usize, 0.5f64, 2usize);
    let s = |m: usize| (TAU * m as f64 / n as f64).sin();
    // Separable field s(i)+s(j)+s(k): ∇² = s''(i)+s''(j)+s''(k) (each a 1-D second difference).
    let u = field3d(n, n, n, |i, j, k| s(i) + s(j) + s(k));
    let lap = apply_3d(
        &laplacian_3d::<f64>(l, l, l, d, d, d, &full()).unwrap(),
        &u,
        l,
    );
    let d2 = |m: usize| (s((m + 1) % n) - 2.0 * s(m) + s((m + n - 1) % n)) / (d * d);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let want = d2(i) + d2(j) + d2(k);
                assert!(
                    (lap[(i * n + j) * n + k] - want).abs() <= 1e-9,
                    "∇² wrong at {i},{j},{k}"
                );
            }
        }
    }
}

#[test]
fn divergence_3d_sums_axis_derivatives() {
    let (n, d, l) = (4usize, 0.5f64, 2usize);
    let s = |m: usize| (TAU * m as f64 / n as f64).sin();
    let fx = field3d(n, n, n, |i, _j, _k| s(i));
    let fy = field3d(n, n, n, |_i, j, _k| s(j));
    let fz = field3d(n, n, n, |_i, _j, k| s(k));
    let t = full();
    let (gx, gy, gz) = (
        gradient_x_3d::<f64>(l, l, l, d, &t).unwrap(),
        gradient_y_3d::<f64>(l, l, l, d, &t).unwrap(),
        gradient_z_3d::<f64>(l, l, l, d, &t).unwrap(),
    );
    let qx = quantize_3d(&fx, &t).unwrap();
    let qy = quantize_3d(&fy, &t).unwrap();
    let qz = quantize_3d(&fz, &t).unwrap();
    let div = divergence_3d(&qx, &qy, &qz, &gx, &gy, &gz, &t).unwrap();
    let out = dequantize_3d(&div, l, l, l).unwrap();
    let cd = |m: usize| (s((m + 1) % n) - s((m + n - 1) % n)) / (2.0 * d);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let want = cd(i) + cd(j) + cd(k);
                assert!(
                    (out.as_slice()[(i * n + j) * n + k] - want).abs() <= 1e-9,
                    "∇·F wrong at {i},{j},{k}"
                );
            }
        }
    }
}

#[test]
fn smooth_input_stays_low_rank() {
    // A smooth separable field's gradient is exact at a modest bond cap (bounded rank).
    let (n, d, l) = (8usize, 0.25f64, 3usize);
    let s = |m: usize| (TAU * m as f64 / n as f64).sin();
    let u = field3d(n, n, n, |i, _j, _k| s(i));
    let capped = Truncation::by_bond(8).unwrap();
    let gx = gradient_x_3d::<f64>(l, l, l, d, &capped).unwrap();
    let q = quantize_3d(&u, &capped).unwrap();
    let out = dequantize_3d(&gx.apply(&q, &capped).unwrap(), l, l, l).unwrap();
    for i in 0..n {
        let want = (s((i + 1) % n) - s((i + n - 1) % n)) / (2.0 * d);
        assert!(
            (out.as_slice()[(i * n) * n] - want).abs() <= 1e-9,
            "bounded-rank gradient wrong at {i}"
        );
    }
}

#[test]
fn quantize_3d_rejects_bad_shape() {
    // Not 3-D.
    let two_d = CausalTensor::new(vec![0.0; 16], vec![4, 4]).unwrap();
    assert!(quantize_3d(&two_d, &full()).is_err());
    // Non-power-of-two extent.
    let bad = CausalTensor::new(vec![0.0; 4 * 4 * 3], vec![4, 4, 3]).unwrap();
    assert!(quantize_3d(&bad, &full()).is_err());
}
