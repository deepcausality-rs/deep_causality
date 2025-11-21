/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

#[test]
fn test_outer_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 ^ e2 = e12
    let res = e1.outer_product(&e2);
    assert_eq!(res.data()[3], 1.0);

    // e1 ^ e1 = 0
    let res2 = e1.outer_product(&e1);
    assert!(res2.data().iter().all(|&x| x == 0.0));
}

#[test]
fn test_inner_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 . e2 = 0
    let res = e1.inner_product(&e2);
    assert!(res.data().iter().all(|&x| x == 0.0));

    // e1 . e1 = 1
    let res2 = e1.inner_product(&e1);
    assert_eq!(res2.data()[0], 1.0);
}
