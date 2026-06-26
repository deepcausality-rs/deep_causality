/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::{CausalTensor, CausalTensorError, Truncation};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive>(data: &[f64], shape: &[usize]) -> CausalTensor<T> {
    let d: Vec<T> = data.iter().map(|&x| v::<T>(x)).collect();
    CausalTensor::new(d, shape.to_vec()).unwrap()
}

/// Working tolerance scaled by precision: `√ε · 16`.
fn tol<T: RealField + FromPrimitive>() -> T {
    T::epsilon().sqrt() * v::<T>(16.0)
}

fn approx<T: RealField + FromPrimitive>(a: T, b: T) {
    assert!(
        (a - b).abs() <= tol::<T>(),
        "values differ beyond tolerance"
    );
}

/// Reconstructs `U · diag(S) · Vt` (sign/order independent).
fn reconstruct<T: RealField>(
    u: &CausalTensor<T>,
    s: &CausalTensor<T>,
    vt: &CausalTensor<T>,
) -> Vec<T> {
    let m = u.shape()[0];
    let k = u.shape()[1];
    let n = vt.shape()[1];
    let (u, s, vt) = (u.as_slice(), s.as_slice(), vt.as_slice());
    let mut out = vec![T::zero(); m * n];
    for row in 0..m {
        for col in 0..n {
            let mut acc = T::zero();
            for t in 0..k {
                acc += u[row * k + t] * s[t] * vt[t * n + col];
            }
            out[row * n + col] = acc;
        }
    }
    out
}

/// Asserts the columns of an `r × c` matrix are orthonormal (`MᵀM ≈ I`).
fn assert_orthonormal_cols<T: RealField + FromPrimitive>(m: &CausalTensor<T>) {
    let rows = m.shape()[0];
    let cols = m.shape()[1];
    let data = m.as_slice();
    for a in 0..cols {
        for b in 0..cols {
            let mut dot = T::zero();
            for i in 0..rows {
                dot += data[i * cols + a] * data[i * cols + b];
            }
            let expected = if a == b { v::<T>(1.0) } else { v::<T>(0.0) };
            approx::<T>(dot, expected);
        }
    }
}

/// Asserts the rows of an `r × c` matrix are orthonormal (`MMᵀ ≈ I`).
fn assert_orthonormal_rows<T: RealField + FromPrimitive>(m: &CausalTensor<T>) {
    let rows = m.shape()[0];
    let cols = m.shape()[1];
    let data = m.as_slice();
    for a in 0..rows {
        for b in 0..rows {
            let mut dot = T::zero();
            for i in 0..cols {
                dot += data[a * cols + i] * data[b * cols + i];
            }
            let expected = if a == b { v::<T>(1.0) } else { v::<T>(0.0) };
            approx::<T>(dot, expected);
        }
    }
}

fn check_svd<T: RealField + FromPrimitive>() {
    let full = Truncation::<T>::by_bond(1024).unwrap();

    // (1) Diagonal matrix: singular values are |diagonal|, descending.
    let a = tensor::<T>(&[4.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 2.0], &[3, 3]);
    let (u, s, vt) = a.svd_truncated(&full).unwrap();
    assert_eq!(s.shape(), &[3]);
    approx::<T>(s.as_slice()[0], v::<T>(4.0));
    approx::<T>(s.as_slice()[1], v::<T>(3.0));
    approx::<T>(s.as_slice()[2], v::<T>(2.0));
    let recon = reconstruct(&u, &s, &vt);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }
    assert_orthonormal_cols(&u);
    assert_orthonormal_rows(&vt);

    // (2) Known 2×2 with golden-ratio singular values: [[1,1],[0,1]] → φ, 1/φ.
    let a = tensor::<T>(&[1.0, 1.0, 0.0, 1.0], &[2, 2]);
    let (u, s, vt) = a.svd_truncated(&full).unwrap();
    approx::<T>(s.as_slice()[0], v::<T>(1.618_033_988_749_895));
    approx::<T>(s.as_slice()[1], v::<T>(0.618_033_988_749_895));
    let recon = reconstruct(&u, &s, &vt);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }
    assert_orthonormal_cols(&u);

    // (3) Rectangular tall matrix reconstructs.
    let a = tensor::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[3, 2]);
    let (u, s, vt) = a.svd_truncated(&full).unwrap();
    assert_eq!(u.shape(), &[3, 2]);
    assert_eq!(vt.shape(), &[2, 2]);
    let recon = reconstruct(&u, &s, &vt);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }
    assert_orthonormal_cols(&u);

    // (4) Rectangular wide matrix (m < n) exercises the transpose path.
    let a = tensor::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[2, 3]);
    let (u, s, vt) = a.svd_truncated(&full).unwrap();
    assert_eq!(u.shape(), &[2, 2]);
    assert_eq!(vt.shape(), &[2, 3]);
    let recon = reconstruct(&u, &s, &vt);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }
    assert_orthonormal_rows(&vt);

    // (5) Truncation by bond keeps exactly k and tracks the discarded tail.
    let a = tensor::<T>(&[4.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 2.0], &[3, 3]);
    let trunc = Truncation::<T>::by_bond(2).unwrap();
    let (u, s, vt) = a.svd_truncated(&trunc).unwrap();
    assert_eq!(s.shape(), &[2]);
    assert_eq!(u.shape(), &[3, 2]);
    assert_eq!(vt.shape(), &[2, 3]);
    // The reconstruction error equals the dropped singular value (2.0) in Frobenius norm.
    let recon = reconstruct(&u, &s, &vt);
    let mut err_sq = T::zero();
    for (g, e) in recon.iter().zip(a.as_slice()) {
        let d = *g - *e;
        err_sq += d * d;
    }
    approx::<T>(err_sq.sqrt(), v::<T>(2.0));

    // (6) Truncation by tolerance drops the small value.
    let trunc = Truncation::<T>::by_tol(v::<T>(0.5)).unwrap();
    let (_u, s, _vt) = a.svd_truncated(&trunc).unwrap();
    // σ = [4,3,2]; rel gate 0.5·4 = 2.0 keeps all three (2 ≥ 2).
    assert_eq!(s.shape(), &[3]);
    let trunc = Truncation::<T>::by_tol(v::<T>(0.6)).unwrap();
    let (_u, s, _vt) = a.svd_truncated(&trunc).unwrap();
    // rel gate 0.6·4 = 2.4 drops the 2.0.
    assert_eq!(s.shape(), &[2]);
}

#[test]
fn test_svd_truncated_f32() {
    check_svd::<f32>();
}

#[test]
fn test_svd_truncated_f64() {
    check_svd::<f64>();
}

#[test]
fn test_svd_truncated_float106() {
    check_svd::<Float106>();
}

#[test]
fn test_svd_truncated_errors() {
    let trunc = Truncation::<f64>::by_bond(4).unwrap();

    // Not 2-dimensional.
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert!(matches!(
        a.svd_truncated(&trunc),
        Err(CausalTensorError::DimensionMismatch)
    ));

    // Empty dimension.
    let a = CausalTensor::<f64>::new(vec![], vec![0, 3]).unwrap();
    assert!(matches!(
        a.svd_truncated(&trunc),
        Err(CausalTensorError::EmptyTensor)
    ));
}
