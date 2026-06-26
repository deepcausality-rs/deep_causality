/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive>(data: &[f64], shape: &[usize]) -> CausalTensor<T> {
    let d: Vec<T> = data.iter().map(|&x| v::<T>(x)).collect();
    CausalTensor::new(d, shape.to_vec()).unwrap()
}

fn tol<T: RealField + FromPrimitive>() -> T {
    T::epsilon().sqrt() * v::<T>(16.0)
}

fn approx<T: RealField + FromPrimitive>(a: T, b: T) {
    assert!(
        (a - b).abs() <= tol::<T>(),
        "values differ beyond tolerance"
    );
}

/// `Q (m×k) · R (k×n)` reconstruction.
fn matmul<T: RealField>(q: &CausalTensor<T>, r: &CausalTensor<T>) -> Vec<T> {
    let m = q.shape()[0];
    let k = q.shape()[1];
    let n = r.shape()[1];
    let (q, r) = (q.as_slice(), r.as_slice());
    let mut out = vec![T::zero(); m * n];
    for row in 0..m {
        for col in 0..n {
            let mut acc = T::zero();
            for t in 0..k {
                acc += q[row * k + t] * r[t * n + col];
            }
            out[row * n + col] = acc;
        }
    }
    out
}

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

/// Asserts the strictly-lower triangle of a `k × n` matrix is exactly zero.
fn assert_upper_triangular<T: RealField>(r: &CausalTensor<T>) {
    let k = r.shape()[0];
    let n = r.shape()[1];
    let data = r.as_slice();
    for i in 0..k {
        for col in 0..n {
            if col < i {
                assert!(data[i * n + col] == T::zero(), "R is not upper-triangular");
            }
        }
    }
}

fn check_qr<T: RealField + FromPrimitive>() {
    // Tall matrix.
    let a = tensor::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[3, 2]);
    let (q, r) = a.qr().unwrap();
    assert_eq!(q.shape(), &[3, 2]);
    assert_eq!(r.shape(), &[2, 2]);
    assert_orthonormal_cols(&q);
    assert_upper_triangular(&r);
    let recon = matmul(&q, &r);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }

    // Square matrix.
    let a = tensor::<T>(&[2.0, -1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 2.0, 4.0], &[3, 3]);
    let (q, r) = a.qr().unwrap();
    assert_eq!(q.shape(), &[3, 3]);
    assert_orthonormal_cols(&q);
    assert_upper_triangular(&r);
    let recon = matmul(&q, &r);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }

    // Wide matrix (k = min(m, n) = m).
    let a = tensor::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[2, 3]);
    let (q, r) = a.qr().unwrap();
    assert_eq!(q.shape(), &[2, 2]);
    assert_eq!(r.shape(), &[2, 3]);
    assert_orthonormal_cols(&q);
    assert_upper_triangular(&r);
    let recon = matmul(&q, &r);
    for (g, e) in recon.iter().zip(a.as_slice()) {
        approx::<T>(*g, *e);
    }
}

#[test]
fn test_qr_f32() {
    check_qr::<f32>();
}

#[test]
fn test_qr_f64() {
    check_qr::<f64>();
}

#[test]
fn test_qr_float106() {
    check_qr::<Float106>();
}

#[test]
fn test_qr_errors() {
    // Not 2-dimensional.
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert!(matches!(a.qr(), Err(CausalTensorError::DimensionMismatch)));

    // Empty dimension.
    let a = CausalTensor::<f64>::new(vec![], vec![0, 3]).unwrap();
    assert!(matches!(a.qr(), Err(CausalTensorError::EmptyTensor)));
}
