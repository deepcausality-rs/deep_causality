/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};

#[test]
fn test_to_matrix_identity() {
    // Create a scalar multivector (1 + 0*e1 + 0*e2 + 0*e12)
    let metric = Metric::from_signature(2, 0, 0);
    let mv = CausalMultiVector::<f64>::scalar(1.0, metric);

    let matrix = mv.to_matrix();
    let data = matrix.to_vec();

    // Should be 2x2 identity matrix: [[1, 0], [0, 1]]
    assert!((data[0] - 1.0).abs() < 1e-10); // (0,0)
    assert!(data[1].abs() < 1e-10); // (0,1)
    assert!(data[2].abs() < 1e-10); // (1,0)
    assert!((data[3] - 1.0).abs() < 1e-10); // (1,1)
}

#[test]
fn test_from_matrix_identity() {
    let metric = Metric::from_signature(2, 0, 0);

    // Create identity matrix
    let matrix = deep_causality_tensor::CausalTensor::<f64>::from_shape_fn(&[2, 2], |idx| {
        if idx[0] == idx[1] { 1.0 } else { 0.0 }
    });

    let mv = CausalMultiVector::<f64>::from_matrix(matrix, metric);

    // Should be scalar 1
    assert!(
        (mv.data()[0] - 1.0).abs() < 1e-6,
        "Scalar should be 1.0, got {}",
        mv.data()[0]
    );

    // Other components should be small
    for i in 1..4 {
        assert!(
            mv.data()[i].abs() < 1e-6,
            "Component {} should be 0, got {}",
            i,
            mv.data()[i]
        );
    }
}

#[test]
fn test_get_gamma_matrix_identity() {
    let metric = Metric::from_signature(2, 0, 0);
    let mv = CausalMultiVector::<f64>::scalar(1.0, metric);

    // Gamma_0 for blade 0 should be identity
    let gamma = mv.get_gamma_matrix(0);
    let data = gamma.to_vec();

    assert!((data[0] - 1.0).abs() < 1e-10);
    assert!(data[1].abs() < 1e-10);
    assert!(data[2].abs() < 1e-10);
    assert!((data[3] - 1.0).abs() < 1e-10);
}
