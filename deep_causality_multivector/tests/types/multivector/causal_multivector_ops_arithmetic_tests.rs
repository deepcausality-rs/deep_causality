/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{CausalMultiVector, Metric};

#[test]
fn test_add_sub() {
    let m = Metric::Euclidean(2);
    let v1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let v2 = CausalMultiVector::new(vec![10.0, 20.0, 30.0, 40.0], m).unwrap();

    let sum = v1.clone() + v2.clone();
    assert_eq!(sum.data(), &vec![11.0, 22.0, 33.0, 44.0]);

    let diff = v2 - v1;
    assert_eq!(diff.data(), &vec![9.0, 18.0, 27.0, 36.0]);
}

#[test]
#[should_panic(expected = "Dimension mismatch")]
fn test_add_mismatch_panic() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Euclidean(3);
    let v1 = CausalMultiVector::scalar(1.0, m1);
    let v2 = CausalMultiVector::scalar(1.0, m2);
    let _ = v1 + v2;
}

#[test]
fn test_mul_scalar() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let res = v * 2.0;
    assert_eq!(res.data(), &vec![2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_div_scalar() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![2.0, 4.0, 6.0, 8.0], m).unwrap();
    let res = v / 2.0;
    assert_eq!(res.data(), &vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_geometric_product_euclidean() {
    // Euclidean 2D: e1*e1 = 1, e2*e2 = 1, e1*e2 = e12, e2*e1 = -e12
    let m = Metric::Euclidean(2);

    // e1 (index 1) * e2 (index 2) -> e12 (index 3)
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    let e1e2 = e1.clone() * e2.clone();
    assert_eq!(e1e2.data()[3], 1.0); // e12 component

    // e2 * e1 -> -e12
    let e2e1 = e2 * e1.clone();
    assert_eq!(e2e1.data()[3], -1.0);

    // e1 * e1 -> 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], 1.0);
}

#[test]
fn test_geometric_product_minkowski() {
    // Minkowski 2D: e0^2 = 1, e1^2 = -1
    // e0 is index 1, e1 is index 2
    let m = Metric::Minkowski(2);

    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e0 * e0 = 1
    let e0sq = e0.clone() * e0.clone();
    assert_eq!(e0sq.data()[0], 1.0);

    // e1 * e1 = -1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], -1.0);
}

#[test]
fn test_geometric_product_pga() {
    // PGA 2D: e0^2 = 0, e1^2 = 1
    let m = Metric::PGA(2);

    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e0 * e0 = 0
    let e0sq = e0.clone() * e0;
    assert_eq!(e0sq.data()[0], 0.0);

    // e1 * e1 = 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], 1.0);
}

#[test]
fn test_geometric_product_sparse_high_dim() {
    // Use a dimension > SPARSE_THRESHOLD (e.g., 7)
    let dim = 7;
    let size = 1 << dim; // 128 elements
    let m = Metric::Euclidean(dim);

    // Create sparse vectors: A = e1, B = e2
    let mut data_a = vec![0.0; size];
    data_a[1] = 1.0; // e1
    let a = CausalMultiVector::new(data_a, m).unwrap();

    let mut data_b = vec![0.0; size];
    data_b[2] = 1.0; // e2
    let b = CausalMultiVector::new(data_b, m).unwrap();

    // e1 * e2 = e12 (index 3)
    let product = a * b;

    // The result should be sparse, with only e12 (index 3) having a coefficient of 1.0
    assert_eq!(product.data()[3], 1.0);
    // All other coefficients should be zero
    for (idx, &val) in product.data().iter().enumerate() {
        if idx != 3 {
            assert_eq!(val, 0.0);
        }
    }

    // Test with e1 * e1 = 1
    let mut data_c = vec![0.0; size];
    data_c[1] = 1.0; // e1
    let c = CausalMultiVector::new(data_c, m).unwrap();
    let c_sq = c.clone() * c;
    assert_eq!(c_sq.data()[0], 1.0); // Scalar part should be 1.0
    for (idx, &val) in c_sq.data().iter().enumerate() {
        if idx != 0 {
            assert_eq!(val, 0.0);
        }
    }
}
