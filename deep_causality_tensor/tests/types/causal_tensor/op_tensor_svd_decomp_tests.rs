/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};

// Reconstructs A from U (m x k), S (k), Vt (k x n) and compares to expected,
// without relying on a particular sign/ordering convention of the factors.
fn reconstruct(u: &CausalTensor<f64>, s: &CausalTensor<f64>, vt: &CausalTensor<f64>) -> Vec<f64> {
    let m = u.shape()[0];
    let k = u.shape()[1];
    let n = vt.shape()[1];
    let u = u.as_slice();
    let s = s.as_slice();
    let vt = vt.as_slice();

    let mut out = vec![0.0; m * n];
    for (row, out_row) in out.chunks_mut(n).enumerate() {
        for (col, cell) in out_row.iter_mut().enumerate() {
            let mut acc = 0.0;
            for t in 0..k {
                acc += u[row * k + t] * s[t] * vt[t * n + col];
            }
            *cell = acc;
        }
    }
    out
}

fn assert_slice_approx(a: &[f64], b: &[f64], eps: f64) {
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert!((x - y).abs() < eps, "{x} !~ {y}");
    }
}

#[test]
fn test_svd_diagonal_matrix() {
    // A diagonal matrix has its diagonal entries as singular values.
    let a = CausalTensor::new(vec![3.0_f64, 0.0, 0.0, 2.0], vec![2, 2]).unwrap();
    let (u, s, vt) = a.svd().unwrap();

    assert_eq!(u.shape(), &[2, 2]);
    assert_eq!(s.shape(), &[2]);
    assert_eq!(vt.shape(), &[2, 2]);

    // Largest singular value should be 3.0 (power iteration finds dominant first).
    assert!((s.as_slice()[0] - 3.0).abs() < 1e-6);

    let recon = reconstruct(&u, &s, &vt);
    assert_slice_approx(&recon, a.as_slice(), 1e-4);
}

#[test]
fn test_svd_square_reconstruction() {
    let a = CausalTensor::new(vec![4.0_f64, 0.0, 3.0, -5.0], vec![2, 2]).unwrap();
    let (u, s, vt) = a.svd().unwrap();
    let recon = reconstruct(&u, &s, &vt);
    assert_slice_approx(&recon, a.as_slice(), 1e-3);
}

#[test]
fn test_svd_tall_matrix() {
    // m > n: U is m x k, Vt is k x n with k = n.
    let a = CausalTensor::new(vec![1.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0], vec![3, 2]).unwrap();
    let (u, s, vt) = a.svd().unwrap();
    assert_eq!(u.shape(), &[3, 2]);
    assert_eq!(s.shape(), &[2]);
    assert_eq!(vt.shape(), &[2, 2]);

    let recon = reconstruct(&u, &s, &vt);
    assert_slice_approx(&recon, a.as_slice(), 1e-3);
}

#[test]
fn test_svd_wide_matrix() {
    // m < n: k = m.
    let a = CausalTensor::new(vec![1.0_f64, 2.0, 3.0, 0.0, 1.0, 0.0], vec![2, 3]).unwrap();
    let (u, s, vt) = a.svd().unwrap();
    assert_eq!(u.shape(), &[2, 2]);
    assert_eq!(s.shape(), &[2]);
    assert_eq!(vt.shape(), &[2, 3]);

    let recon = reconstruct(&u, &s, &vt);
    assert_slice_approx(&recon, a.as_slice(), 1e-3);
}

#[test]
fn test_svd_zero_matrix() {
    // A zero matrix yields zero singular values; exercises the new_sigma == 0 break.
    let a = CausalTensor::new(vec![0.0_f64, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();
    let (_u, s, _vt) = a.svd().unwrap();
    for &sv in s.as_slice() {
        assert!(sv.abs() < 1e-9);
    }
}

#[test]
fn test_svd_non_2d_error() {
    let a = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let err = a.svd().unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);
}

#[test]
fn test_svd_3d_error() {
    let a = CausalTensor::new(vec![1.0_f64; 8], vec![2, 2, 2]).unwrap();
    let err = a.svd().unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);
}
