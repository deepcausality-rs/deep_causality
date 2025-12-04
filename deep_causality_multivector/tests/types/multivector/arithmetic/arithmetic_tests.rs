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
fn test_ref_add_sub() {
    let m = Metric::Euclidean(2);
    let v1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let v2 = CausalMultiVector::new(vec![10.0, 20.0, 30.0, 40.0], m).unwrap();

    let sum = &v1 + &v2;
    assert_eq!(sum.data(), &vec![11.0, 22.0, 33.0, 44.0]);

    let diff = &v2 - &v1;
    assert_eq!(diff.data(), &vec![9.0, 18.0, 27.0, 36.0]);
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_ref_add_mismatch_panic() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Euclidean(3);
    let v1 = CausalMultiVector::scalar(1.0, m1);
    let v2 = CausalMultiVector::scalar(1.0, m2);
    let _ = &v1 + &v2;
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
fn test_ref_mul_scalar() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let res = &v * 2.0;
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
fn test_ref_div_scalar() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![2.0, 4.0, 6.0, 8.0], m).unwrap();
    let res = &v / 2.0;
    assert_eq!(res.data(), &vec![1.0, 2.0, 3.0, 4.0]);
}
