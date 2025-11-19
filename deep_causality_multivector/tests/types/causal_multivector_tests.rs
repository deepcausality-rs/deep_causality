/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::types::metric::Metric;

#[test]
fn test_new_valid() {
    let m = Metric::Euclidean(2); // dim 2 -> size 4
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let mv = CausalMultiVector::new(data.clone(), m).unwrap();
    assert_eq!(mv.data, data);
    assert_eq!(mv.metric, m);
}

#[test]
fn test_new_invalid_length() {
    let m = Metric::Euclidean(2); // dim 2 -> size 4
    let data = vec![1.0, 2.0, 3.0]; // Too short
    let res = CausalMultiVector::new(data, m);
    assert!(res.is_err());
}

#[test]
fn test_scalar_constructor() {
    let m = Metric::Euclidean(2);
    let mv = CausalMultiVector::scalar(5.0, m);
    assert_eq!(mv.data[0], 5.0);
    assert_eq!(mv.data[1], 0.0);
}

#[test]
fn test_pseudoscalar() {
    let m = Metric::Euclidean(2);
    let mv: CausalMultiVector<f64> = CausalMultiVector::pseudoscalar(m);
    assert_eq!(mv.data[3], 1.0); // Index 3 is e1^e2 (11 binary)
    assert_eq!(mv.data[0], 0.0);
}

#[test]
fn test_add_sub() {
    let m = Metric::Euclidean(2);
    let v1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let v2 = CausalMultiVector::new(vec![10.0, 20.0, 30.0, 40.0], m).unwrap();

    let sum = v1.clone() + v2.clone();
    assert_eq!(sum.data, vec![11.0, 22.0, 33.0, 44.0]);

    let diff = v2 - v1;
    assert_eq!(diff.data, vec![9.0, 18.0, 27.0, 36.0]);
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
    assert_eq!(res.data, vec![2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_div_scalar() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![2.0, 4.0, 6.0, 8.0], m).unwrap();
    let res = v / 2.0;
    assert_eq!(res.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_geometric_product_euclidean() {
    // Euclidean 2D: e1*e1 = 1, e2*e2 = 1, e1*e2 = e12, e2*e1 = -e12
    let m = Metric::Euclidean(2);

    // e1 (index 1) * e2 (index 2) -> e12 (index 3)
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    let e1e2 = e1.clone() * e2.clone();
    assert_eq!(e1e2.data[3], 1.0); // e12 component

    // e2 * e1 -> -e12
    let e2e1 = e2 * e1.clone();
    assert_eq!(e2e1.data[3], -1.0);

    // e1 * e1 -> 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data[0], 1.0);
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
    assert_eq!(e0sq.data[0], 1.0);

    // e1 * e1 = -1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data[0], -1.0);
}

#[test]
fn test_geometric_product_pga() {
    // PGA 2D: e0^2 = 0, e1^2 = 1
    let m = Metric::PGA(2);

    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e0 * e0 = 0
    let e0sq = e0.clone() * e0;
    assert_eq!(e0sq.data[0], 0.0);

    // e1 * e1 = 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data[0], 1.0);
}

#[test]
fn test_reversion() {
    let m = Metric::Euclidean(2);
    // A = 1 + 2e1 + 3e2 + 4e12
    // ~A = 1 + 2e1 + 3e2 - 4e12 (bivectors reverse sign)
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let rev = v.reversion();
    assert_eq!(rev.data, vec![1.0, 2.0, 3.0, -4.0]);
}

#[test]
fn test_squared_magnitude() {
    let m = Metric::Euclidean(2);
    // e1 -> mag^2 = 1
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    assert_eq!(e1.squared_magnitude(), 1.0);

    // e1 + e2 -> (e1+e2)(e1+e2) = 1 + e1e2 + e2e1 + 1 = 2
    let v = CausalMultiVector::new(vec![0.0, 1.0, 1.0, 0.0], m).unwrap();
    assert_eq!(v.squared_magnitude(), 2.0);
}

#[test]
fn test_inverse() {
    let m = Metric::Euclidean(2);
    // e1^-1 = e1 (since e1^2 = 1)
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let inv = e1.inverse().unwrap();
    assert_eq!(inv.data, e1.data);

    // Zero vector -> error
    let zero = CausalMultiVector::scalar(0.0, m);
    assert!(zero.inverse().is_err());
}

#[test]
fn test_dual() {
    let m = Metric::Euclidean(2);
    // I = e12
    // e1 * = e1 * I^-1 = e1 * (-e12) = -e1 e1 e2 = -e2
    // Wait: I = e1e2. I^2 = -1. I^-1 = -I = -e1e2.
    // e1 * (-e1e2) = -e1 e1 e2 = -e2.

    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let dual = e1.dual().unwrap();

    // e2 is index 2. -e2 means index 2 is -1.
    assert_eq!(dual.data[2], -1.0);
}

#[test]
fn test_outer_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 ^ e2 = e12
    let res = e1.outer_product(&e2);
    assert_eq!(res.data[3], 1.0);

    // e1 ^ e1 = 0
    let res2 = e1.outer_product(&e1);
    assert!(res2.data.iter().all(|&x| x == 0.0));
}

#[test]
fn test_inner_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 . e2 = 0
    let res = e1.inner_product(&e2);
    assert!(res.data.iter().all(|&x| x == 0.0));

    // e1 . e1 = 1
    let res2 = e1.inner_product(&e1);
    assert_eq!(res2.data[0], 1.0);
}

#[test]
fn test_grade_projection() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    // 1 (gr0) + 2e1 (gr1) + 3e2 (gr1) + 4e12 (gr2)

    let g0 = v.grade_projection(0);
    assert_eq!(g0.data[0], 1.0);
    assert_eq!(g0.data[1], 0.0);

    let g1 = v.grade_projection(1);
    assert_eq!(g1.data[1], 2.0);
    assert_eq!(g1.data[2], 3.0);
    assert_eq!(g1.data[0], 0.0);
}
