/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_topology::{CurvatureSymmetry, CurvatureTensor};

#[test]
fn test_flat_tensor() {
    let flat: CurvatureTensor<f64, f64, f64, f64> = CurvatureTensor::flat(4);
    assert!(flat.is_flat());
    assert_eq!(flat.dim(), 4);
    assert_eq!(flat.ricci_scalar(), 0.0);
}

#[test]
fn test_contraction_flat() {
    let flat: CurvatureTensor<f64, f64, f64, f64> = CurvatureTensor::flat(4);
    let u = vec![1.0, 0.0, 0.0, 0.0];
    let v = vec![0.0, 1.0, 0.0, 0.0];
    let w = vec![0.0, 0.0, 1.0, 0.0];
    let result = flat.contract(&u, &v, &w);
    assert!(result.iter().all(|&x| x.abs() < f64::EPSILON));
}

#[test]
fn test_from_generator() {
    let tensor: CurvatureTensor<f64, f64, f64, f64> = CurvatureTensor::from_generator(
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
    let tensor: CurvatureTensor<f64, f64, f64, f64> = CurvatureTensor::from_generator(
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
    let tensor: CurvatureTensor<f64, f64, f64, f64> = CurvatureTensor::from_generator(
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
