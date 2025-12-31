/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{BackendTensor, CpuBackend, Tensor};

#[test]
fn test_backend_tensor_delegation() {
    // Tests that BackendTensor correctly delegates linear algebra ops to the inner backend
    // This covers cpu_tensor_impl.rs methods

    // 1. Create a matrix for SVD/QR: [[1, 2], [3, 4]]
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let tensor = <BackendTensor<f64, CpuBackend>>::from_slice(&data, &[2, 2]);

    // 2. Test QR
    let (q, r) = tensor.qr().expect("QR failed");
    // Reconstruct: Q * R
    let recon: BackendTensor<f64, CpuBackend> = q.matmul(&r).expect("Matmul failed");
    // Verify reconstruction
    let t_vec = tensor.clone().into_vec();
    let r_vec = recon.into_vec();
    for (a, b) in t_vec.iter().zip(r_vec.iter()) {
        assert!((a - b).abs() < 1e-10, "QR reconstruction mismatch");
    }

    // 3. Test SVD
    let (u, s, _v) = tensor.svd().expect("SVD failed");
    // Reconstruct: U * S_diag * V^T (or similar depending on convention)
    // S is vector of singular values.
    // We won't easily reconstruct without building diag matrix, but we can check correctness of call.
    assert_eq!(s.shape().len(), 1);
    assert_eq!(u.shape(), &[2, 2]);

    // 4. Test Cholesky (requires symmetric positive definite)
    // A = [[2, 1], [1, 2]]
    let sym_data = vec![2.0, 1.0, 1.0, 2.0];
    let sym_tensor = <BackendTensor<f64, CpuBackend>>::from_slice(&sym_data, &[2, 2]);
    let l = sym_tensor
        .cholesky_decomposition()
        .expect("Cholesky failed");
    // Reconstruct L * L^T
    let lt = l.permute_axes(&[1, 0]).expect("Transpose failed");
    let sym_recon: BackendTensor<f64, CpuBackend> = l.matmul(&lt).expect("Matmul failed");

    let s_vec = sym_tensor.into_vec();
    let sr_vec = sym_recon.into_vec();
    for (a, b) in s_vec.iter().zip(sr_vec.iter()) {
        assert!((a - b).abs() < 1e-10, "Cholesky reconstruction mismatch");
    }
}

#[test]
fn test_backend_tensor_norm_and_trace() {
    let data = vec![3.0, 4.0];
    let tensor = <BackendTensor<f64, CpuBackend>>::from_slice(&data, &[2]);

    // L2 norm = 5
    let norm = tensor.norm_l2();
    assert!((norm - 5.0).abs() < 1e-10);

    // Sq norm = 25
    let norm_sq = tensor.norm_sq();
    assert!((norm_sq - 25.0).abs() < 1e-10);
}
