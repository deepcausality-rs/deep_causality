/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, DefaultMultivectorBackend, Metric};
use deep_causality_tensor::TensorBackend;

#[test]
fn test_matrix_conversion_cycle() {
    // 2D Euclidean: 4 blades (1, e1, e2, e12). Matrix dim = 2^(2/2) = 2.
    // We use DefaultMultivectorBackend which usually maps to CpuBackend
    // We need to ensure f64 is supported or use f32 if that's the default.
    // The previous tests used f32, let's stick to f32 to be safe or check backend.
    // Actually, DefaultMultivectorBackend is typically aliases to CpuBackend.

    let metric = Metric::Euclidean(2);
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let mv = CausalMultiVector::<f64>::unchecked(data.clone(), metric);

    // 1. Convert to Matrix
    let matrix = mv.to_matrix_on_backend::<DefaultMultivectorBackend>();

    // Check dimensions: [2, 2]
    assert_eq!(DefaultMultivectorBackend::shape(&matrix), &[2, 2]);

    // 2. Convert back to coefficients
    let recovered = CausalMultiVector::<f64>::from_matrix_on_backend::<DefaultMultivectorBackend>(
        matrix, metric,
    );

    // 3. Compare
    let original_data = mv.data();
    let recovered_data = recovered.data();

    assert_eq!(
        original_data.len(),
        recovered_data.len(),
        "Recovered data length mismatch"
    );

    for (i, (o, r)) in original_data.iter().zip(recovered_data.iter()).enumerate() {
        assert!(
            (o - r).abs() < 1e-10,
            "Mismatch at index {}: original {}, recovered {}",
            i,
            o,
            r
        );
    }
}

#[test]
fn test_get_gamma_matrix() {
    // 3D Euclidean (Pauli matrices + identity structure)
    // dim = 3. num_blades = 8. matrix_dim = 2^(ceil(3/2)) = 2^2 = 4.
    // 4x4 matrices (Dirac-like)
    let metric = Metric::Euclidean(3);
    let mv = CausalMultiVector::<f64>::new(vec![0.0; 8], metric).unwrap();

    // Get gamma_0 (scalar basis 1) -> Identity
    let g0 = mv.get_gamma_matrix::<DefaultMultivectorBackend>(0);
    assert_eq!(DefaultMultivectorBackend::shape(&g0), &[4, 4]);

    // Identity should be diagonal 1
    // We can't easily iterate backend tensor without converting to data
    // Assuming CpuBackend behavior for to_vec calculation or specific check
    // Instead, let's just check shape and maybe trace
    // Trace of Identity(4) = 4
    // We get the data back to verify trace
    let g0_vec = DefaultMultivectorBackend::into_vec(g0);
    // g0 is [4,4], row major usually. Diagonals at 0, 5, 10, 15
    let trace: f64 = g0_vec[0] + g0_vec[5] + g0_vec[10] + g0_vec[15];
    assert!((trace - 4.0).abs() < 1e-10);

    // Get gamma_1 (e1)
    let g1 = mv.get_gamma_matrix::<DefaultMultivectorBackend>(1);
    assert_eq!(DefaultMultivectorBackend::shape(&g1), &[4, 4]);

    // Basis vectors in this representation are usually traceless (except scalar)
    let g1_vec = DefaultMultivectorBackend::into_vec(g1);
    let trace_g1: f64 = g1_vec[0] + g1_vec[5] + g1_vec[10] + g1_vec[15];
    assert!(
        trace_g1.abs() < 1e-10,
        "Gamma vector should be traceless, got {}",
        trace_g1
    );
}

#[test]
fn test_matrix_conversion_cycle_3d() {
    // 3D: 8 blades, 4x4 matrix
    let metric = Metric::Euclidean(3);
    let mut data = vec![0.0; 8];
    for (i, val) in data.iter_mut().enumerate() {
        *val = (i as f64) * 0.5;
    }
    let mv = CausalMultiVector::<f64>::unchecked(data.clone(), metric);

    let matrix = mv.to_matrix_on_backend::<DefaultMultivectorBackend>();
    assert_eq!(DefaultMultivectorBackend::shape(&matrix), &[4, 4]);

    let recovered = CausalMultiVector::<f64>::from_matrix_on_backend::<DefaultMultivectorBackend>(
        matrix, metric,
    );

    let original_data = mv.data();
    let recovered_data = recovered.data();

    for (i, (o, r)) in original_data.iter().zip(recovered_data.iter()).enumerate() {
        assert!(
            (o - r).abs() < 1e-10,
            "Mismatch at index {} in 3D: original {}, recovered {}",
            i,
            o,
            r
        );
    }
}
