/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex kernels: the conjugate-aware SVD and QR over `Complex<f64>` — the genuine
//! Hermitian SVD (unitary `U`/`V`, real singular values, `U·diag(S)·Vᴴ = A`) and the complex
//! Householder QR (unitary `Q`, `Q·R = A`).

use deep_causality_num::{Complex, ConjugateScalar, Zero};
use deep_causality_tensor::{CausalTensor, Truncation};

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

/// Modulus of a complex number.
fn cabs(z: C) -> f64 {
    z.modulus_squared().sqrt()
}

const TOL: f64 = 1e-9;

#[test]
fn test_complex_svd_reconstruction_unitarity_real_singular_values() {
    let (m, n) = (4usize, 3usize);
    let data: Vec<C> = (0..m * n)
        .map(|i| c((i as f64 * 0.7).sin() + 0.3, (i as f64 * 0.4).cos() - 0.2))
        .collect();
    let a = CausalTensor::new(data.clone(), vec![m, n]).unwrap();

    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let (u, s, vt) = a.svd_truncated(&trunc).unwrap();
    let k = s.shape()[0];
    let (ud, sv, vd) = (u.as_slice(), s.as_slice(), vt.as_slice());

    // Singular values are real, non-negative, and non-increasing.
    for w in sv.windows(2) {
        assert!(w[0] >= w[1] - 1e-12, "singular values not sorted");
    }
    for &x in sv {
        assert!(x >= -1e-12, "negative singular value");
    }

    // Reconstruction: U · diag(S) · Vᴴ ≈ A (S real, U/Vt complex).
    for row in 0..m {
        for col in 0..n {
            let mut acc = C::zero();
            for t in 0..k {
                acc += ud[row * k + t] * c(sv[t], 0.0) * vd[t * n + col];
            }
            assert!(cabs(acc - data[row * n + col]) <= TOL, "reconstruction off");
        }
    }

    // U has orthonormal columns under the Hermitian inner product: Uᴴ U = I.
    let uk = u.shape()[1];
    for a_ in 0..uk {
        for b_ in 0..uk {
            let mut acc = C::zero();
            for i in 0..m {
                acc += ud[i * uk + a_].conjugate() * ud[i * uk + b_];
            }
            let expect = if a_ == b_ { 1.0 } else { 0.0 };
            assert!(cabs(acc - c(expect, 0.0)) <= TOL, "U not unitary");
        }
    }
}

#[test]
fn test_complex_qr_unitary_and_reconstruction() {
    let (m, n) = (4usize, 3usize);
    let data: Vec<C> = (0..m * n)
        .map(|i| c((i as f64 * 0.5).cos() + 0.4, (i as f64 * 0.9).sin() - 0.3))
        .collect();
    let a = CausalTensor::new(data.clone(), vec![m, n]).unwrap();

    let (q, r) = a.qr().unwrap();
    let k = q.shape()[1];
    let (qd, rd) = (q.as_slice(), r.as_slice());

    // Q has orthonormal columns: Qᴴ Q = I.
    for a_ in 0..k {
        for b_ in 0..k {
            let mut acc = C::zero();
            for i in 0..m {
                acc += qd[i * k + a_].conjugate() * qd[i * k + b_];
            }
            let expect = if a_ == b_ { 1.0 } else { 0.0 };
            assert!(cabs(acc - c(expect, 0.0)) <= TOL, "Q not unitary");
        }
    }

    // Q · R ≈ A.
    for row in 0..m {
        for col in 0..n {
            let mut acc = C::zero();
            for t in 0..k {
                acc += qd[row * k + t] * rd[t * n + col];
            }
            assert!(
                cabs(acc - data[row * n + col]) <= TOL,
                "QR reconstruction off"
            );
        }
    }

    // R is upper-triangular.
    for i in 0..k {
        for j in 0..i.min(n) {
            assert!(cabs(rd[i * n + j]) <= 1e-12, "R not upper-triangular");
        }
    }
}

#[test]
fn test_complex_svd_wide_matrix() {
    // The wide path (m < n) uses the conjugate transpose internally.
    let (m, n) = (2usize, 5usize);
    let data: Vec<C> = (0..m * n)
        .map(|i| c((i as f64 * 1.1).sin(), (i as f64 * 0.6).cos()))
        .collect();
    let a = CausalTensor::new(data.clone(), vec![m, n]).unwrap();
    let trunc = Truncation::<f64>::by_bond(64).unwrap();
    let (u, s, vt) = a.svd_truncated(&trunc).unwrap();
    let k = s.shape()[0];
    let (ud, sv, vd) = (u.as_slice(), s.as_slice(), vt.as_slice());
    for row in 0..m {
        for col in 0..n {
            let mut acc = C::zero();
            for t in 0..k {
                acc += ud[row * k + t] * c(sv[t], 0.0) * vd[t * n + col];
            }
            assert!(
                cabs(acc - data[row * n + col]) <= TOL,
                "wide reconstruction off"
            );
        }
    }
}
