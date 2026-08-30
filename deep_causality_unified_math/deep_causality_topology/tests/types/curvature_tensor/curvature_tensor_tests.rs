/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

// =============================================================================
// Constructor & getter tests
// =============================================================================

#[test]
fn test_new_constructs_with_shape() {
    let dim = 2;
    let total = dim * dim * dim * dim;
    let data: Vec<f64> = (0..total).map(|i| i as f64).collect();
    let components =
        deep_causality_tensor::CausalTensor::new(data, vec![dim, dim, dim, dim]).unwrap();

    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::new(
        components,
        Metric::Euclidean(2),
        CurvatureSymmetry::Riemann,
        dim,
    );
    assert_eq!(t.dim(), dim);
    assert_eq!(t.symmetry(), CurvatureSymmetry::Riemann);
    assert_eq!(t.metric(), Metric::Euclidean(2));
    assert_eq!(t.components().shape(), vec![2, 2, 2, 2]);
}

#[test]
#[should_panic(expected = "CurvatureTensor components must have shape")]
fn test_new_panics_on_shape_mismatch() {
    // Build a tensor with wrong shape and verify the assert fires.
    let data: Vec<f64> = vec![0.0; 8];
    let components = deep_causality_tensor::CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let _ = CurvatureTensor::<f64, f64, f64, f64, f64>::new(
        components,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        2,
    );
}

#[test]
fn test_flat_with_metric_preserves_metric_choice() {
    let t: CurvatureTensor<f64, f64, f64, f64, f64> =
        CurvatureTensor::flat_with_metric(3, Metric::Euclidean(3));
    assert!(t.is_flat());
    assert_eq!(t.metric(), Metric::Euclidean(3));
    assert_eq!(t.dim(), 3);
}

#[test]
fn test_symmetry_variants_distinct() {
    // Exercise PartialEq/Eq on the enum + the symmetry getter for each variant.
    let t_riemann: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(2);
    assert_eq!(t_riemann.symmetry(), CurvatureSymmetry::Riemann);

    let t_weyl: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::Weyl,
        |_, _, _, _| 0.0,
    );
    assert_eq!(t_weyl.symmetry(), CurvatureSymmetry::Weyl);

    let t_ricci: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::Ricci,
        |_, _, _, _| 0.0,
    );
    assert_eq!(t_ricci.symmetry(), CurvatureSymmetry::Ricci);

    assert_ne!(CurvatureSymmetry::Ricci, CurvatureSymmetry::Weyl);
}

// =============================================================================
// kretschmann_scalar tests
// =============================================================================

#[test]
fn test_kretschmann_scalar_flat_is_zero() {
    let flat: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(4);
    let k = flat.kretschmann_scalar();
    assert!(k.abs() < 1e-12);
}

#[test]
fn test_kretschmann_scalar_with_single_component() {
    // Only R^0_101 = 1; sum of squares = 1.
    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
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
    let k = t.kretschmann_scalar();
    assert!((k - 1.0).abs() < 1e-12);
}

#[test]
fn test_kretschmann_scalar_with_metric_invalid_size_returns_zero() {
    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(2);
    // dim*dim = 4 elements expected; provide 3.
    let inv_metric = vec![1.0, 0.0, 0.0];
    let k = t.kretschmann_scalar_with_metric(&inv_metric);
    assert_eq!(k, 0.0);
}

#[test]
fn test_kretschmann_scalar_with_metric_flat_is_zero() {
    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(2);
    let inv_metric = vec![1.0, 0.0, 0.0, 1.0]; // 2x2 identity
    let k = t.kretschmann_scalar_with_metric(&inv_metric);
    assert!(k.abs() < 1e-10);
}

// =============================================================================
// einstein_tensor tests
// =============================================================================

#[test]
fn test_einstein_tensor_flat_is_zero() {
    let flat: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(4);
    let g = flat.einstein_tensor();
    assert_eq!(g.len(), 16);
    for v in g {
        assert!(v.abs() < 1e-12);
    }
}

#[test]
fn test_einstein_tensor_size_3d() {
    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::flat(3);
    let g = t.einstein_tensor();
    assert_eq!(g.len(), 9);
}

// =============================================================================
// cast() type-conversion
// =============================================================================

#[test]
fn test_cast_preserves_components_and_metadata() {
    let t: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::Riemann,
        |d, a, b, c| (d + a + b + c) as f64,
    );

    // Cast to different phantom parameters (any 4 distinct types).
    let casted: CurvatureTensor<f64, u32, u8, i16, i64> = t.cast();
    assert_eq!(casted.dim(), 2);
    assert_eq!(casted.symmetry(), CurvatureSymmetry::Riemann);
    assert_eq!(casted.metric(), Metric::Euclidean(2));
    // Component R^0_000 = 0, R^1_111 = 4
    assert_eq!(casted.get(0, 0, 0, 0), 0.0);
    assert_eq!(casted.get(1, 1, 1, 1), 4.0);
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
