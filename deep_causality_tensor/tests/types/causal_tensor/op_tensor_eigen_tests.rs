/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};

fn sorted_reals(vals: &[f64]) -> Vec<f64> {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

#[test]
fn test_eigen_hermitian_diagonal() {
    let a = CausalTensor::new(vec![3.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let (vals, _v) = a.eigen_hermitian().unwrap();
    let s = sorted_reals(&vals);
    assert!((s[0] + 1.0).abs() < 1e-12);
    assert!((s[1] - 3.0).abs() < 1e-12);
}

#[test]
fn test_eigen_hermitian_pauli_x() {
    // σx has eigenvalues ±1.
    let a = CausalTensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let (vals, v) = a.eigen_hermitian().unwrap();
    let s = sorted_reals(&vals);
    assert!((s[0] + 1.0).abs() < 1e-12);
    assert!((s[1] - 1.0).abs() < 1e-12);

    // Reconstruction: A = V diag(λ) Vᴴ.
    let n = 2;
    let vt = v.dagger().unwrap();
    let mut lam = vec![0.0; n * n];
    for i in 0..n {
        lam[i * n + i] = vals[i];
    }
    let lam = CausalTensor::new(lam, vec![n, n]).unwrap();
    let recon = v.matmul(&lam).unwrap().matmul(&vt).unwrap();
    for (r, e) in recon.as_slice().iter().zip(a.as_slice()) {
        assert!((r - e).abs() < 1e-10, "reconstruction mismatch");
    }
}

#[test]
fn test_eigen_hermitian_complex() {
    // [[2, i], [-i, 2]] has eigenvalues 1 and 3.
    let a = CausalTensor::new(
        vec![
            Complex::new(2.0, 0.0),
            Complex::new(0.0, 1.0),
            Complex::new(0.0, -1.0),
            Complex::new(2.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let (vals, v) = a.eigen_hermitian().unwrap();
    let mut re: Vec<f64> = vals.iter().map(|c| c.re).collect();
    re.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((re[0] - 1.0).abs() < 1e-12);
    assert!((re[1] - 3.0).abs() < 1e-12);
    // Eigenvalues of a Hermitian matrix are real.
    for c in &vals {
        assert!(c.im.abs() < 1e-12);
    }

    // Reconstruction A = V diag(λ) Vᴴ.
    let n = 2;
    let vt = v.dagger().unwrap();
    let mut lam = vec![Complex::new(0.0, 0.0); n * n];
    for i in 0..n {
        lam[i * n + i] = vals[i];
    }
    let lam = CausalTensor::new(lam, vec![n, n]).unwrap();
    let recon = v.matmul(&lam).unwrap().matmul(&vt).unwrap();
    for (r, e) in recon.as_slice().iter().zip(a.as_slice()) {
        assert!((r.re - e.re).abs() < 1e-10 && (r.im - e.im).abs() < 1e-10);
    }
}

#[test]
fn test_eigen_hermitian_psd_has_nonnegative_spectrum() {
    // Gram matrix G = Mᵀ M is PSD.
    let m = CausalTensor::new(vec![1.0, 2.0, 0.5, -1.0, 3.0, 0.25], vec![3, 2]).unwrap();
    let g = m.dagger().unwrap().matmul(&m).unwrap();
    let (vals, _) = g.eigen_hermitian().unwrap();
    for v in vals {
        assert!(v >= -1e-12, "PSD eigenvalue went negative: {}", v);
    }
}

#[test]
fn test_eigen_hermitian_rejects_bad_shapes() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert_eq!(
        a.eigen_hermitian().unwrap_err(),
        CausalTensorError::DimensionMismatch
    );

    let b = CausalTensor::new(vec![1.0; 6], vec![2, 3]).unwrap();
    assert_eq!(
        b.eigen_hermitian().unwrap_err(),
        CausalTensorError::ShapeMismatch
    );
}

#[test]
fn test_eigen_hermitian_large_finite_norm_still_diagonalizes() {
    // A = x·[[1,1],[1,1]] with x = 1e154: ‖A‖²_F = 4·(1e154)² overflows f64 to
    // +inf, yet A is finite Hermitian with eigenvalues {0, 2x}. The convergence
    // threshold must not become ∞ and break before any rotation — that would
    // return the input diagonal {x, x} unchanged as the "eigenvalues".
    let x = 1e154_f64;
    let a = CausalTensor::new(vec![x, x, x, x], vec![2, 2]).unwrap();
    let (vals, _v) = a.eigen_hermitian().unwrap();
    let s = sorted_reals(&vals);
    // Absolute tolerance ~ε·2x so the near-zero eigenvalue and 2x both check out.
    let atol = 1e140;
    assert!(s[0].abs() < atol, "smallest eigenvalue should be ~0, got {}", s[0]);
    assert!(
        (s[1] - 2.0 * x).abs() < atol,
        "largest eigenvalue should be 2e154, got {}",
        s[1]
    );
}
