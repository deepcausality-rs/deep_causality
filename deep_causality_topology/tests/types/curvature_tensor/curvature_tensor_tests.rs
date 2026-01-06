/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_topology::{CurvatureSymmetry, CurvatureTensor};

#[test]
fn test_flat_tensor() {
    let flat: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(4);
    assert!(flat.is_flat());
    assert_eq!(flat.dim(), 4);
    assert_eq!(flat.ricci_scalar(), 0.0);
}

#[test]
fn test_contraction_flat() {
    let flat: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(4);
    let u = vec![1.0, 0.0, 0.0, 0.0];
    let v = vec![0.0, 1.0, 0.0, 0.0];
    let w = vec![0.0, 0.0, 1.0, 0.0];
    let result = flat.contract(&u, &v, &w);
    assert!(result.iter().all(|&x| x.abs() < f64::EPSILON));
}

#[test]
fn test_from_generator() {
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            if d == 0 && a == 1 && b == 0 && c == 1 {
                1.0
            } else {
                0.0
            }
        },
    );
    assert_eq!(tensor.get(0, 1, 0, 1), 1.0);
    assert_eq!(tensor.get(1, 0, 1, 0), 0.0);
}

#[test]
fn test_ricci_tensor() {
    // Create a tensor with R^0_101 = 1.0
    // Ricci R_μν = R^ρ_μρν
    // R_11 = R^0_101 + R^1_111 (latter 0) = 1.0
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            if d == 0 && a == 1 && b == 0 && c == 1 {
                1.0
            } else {
                0.0
            }
        },
    );

    let ricci = tensor.ricci_tensor();
    // 2x2 matrix: [0,0, 0,1] flattened is [0, 0, 0, 1]?
    // Indices [mu, nu].
    // R_00 = R^0_000 + R^1_010 = 0
    // R_01 = R^0_001 + R^1_011 = 0
    // R_10 = R^0_100 + R^1_110 = 0
    // R_11 = R^0_101 + R^1_111 = 1.0 + 0 = 1.0

    assert_eq!(ricci[3], 1.0); // Index 3 is (1,1) in 2x2
    assert_eq!(ricci[0], 0.0);
}

#[test]
fn test_bianchi_check() {
    // A tensor that violates Bianchi: R_0123 = 1, others 0.
    // Cyclic sum R_0123 + R_0231 + R_0312 = 1 + 0 + 0 = 1 != 0
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        4,
        Metric::Minkowski(4),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            // Using d=0 for simplicity
            if d == 0 && a == 1 && b == 2 && c == 3 {
                1.0
            } else {
                0.0
            }
        },
    );

    let violation = tensor.check_bianchi_identity();
    assert!(violation > 0.0);
}

// =============================================================================
// Weyl Tensor Tests
// =============================================================================

#[test]
fn test_weyl_tensor_flat_space() {
    // In flat space, Weyl tensor should be zero
    let flat: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(4);
    let weyl = flat.weyl_tensor();

    for val in weyl.iter() {
        assert!(val.abs() < 1e-10, "Weyl should be zero in flat space");
    }
}

#[test]
fn test_weyl_tensor_dim_2() {
    // In 2D, Weyl tensor is identically zero
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            if d == 0 && a == 1 && b == 0 && c == 1 {
                1.0
            } else {
                0.0
            }
        },
    );

    let weyl = tensor.weyl_tensor();
    // 2^4 = 16 elements, all should be zero
    assert_eq!(weyl.len(), 16);
    for val in weyl.iter() {
        assert!(val.abs() < 1e-10, "Weyl is zero in 2D");
    }
}

#[test]
fn test_weyl_tensor_size() {
    // Weyl tensor should have dim^4 elements
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(3);
    let weyl = tensor.weyl_tensor();
    assert_eq!(weyl.len(), 81); // 3^4 = 81
}

#[test]
fn test_weyl_tensor_4d_non_trivial() {
    // Verify Weyl tensor computation works for a non-trivial Riemann tensor
    // Just ensure it computes without error and has finite values
    let tensor: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        4,
        Metric::Minkowski(4),
        CurvatureSymmetry::Riemann,
        |d, a, b, c| {
            // Some non-zero Riemann components
            if d == 0 && a == 1 && b == 2 && c == 3 {
                1.0
            } else if d == 1 && a == 0 && b == 2 && c == 3 {
                -1.0 // Antisymmetry
            } else {
                0.0
            }
        },
    );

    let weyl = tensor.weyl_tensor();

    // Should have 256 elements (4^4)
    assert_eq!(weyl.len(), 256);

    // All values should be finite
    for val in weyl.iter() {
        assert!(val.is_finite(), "Weyl tensor should have finite values");
    }
}
