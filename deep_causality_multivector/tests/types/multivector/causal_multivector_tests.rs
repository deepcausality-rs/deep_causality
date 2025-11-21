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
    assert_eq!(mv.data(), &data);
    assert_eq!(mv.metric(), m);
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
    assert_eq!(mv.data()[0], 5.0);
    assert_eq!(mv.data()[1], 0.0);
}

#[test]
fn test_pseudoscalar() {
    let m = Metric::Euclidean(2);
    let mv: CausalMultiVector<f64> = CausalMultiVector::pseudoscalar(m);
    assert_eq!(mv.data()[3], 1.0); // Index 3 is e1^e2 (11 binary)
    assert_eq!(mv.data()[0], 0.0);
}

#[test]
fn test_get() {
    let m = Metric::Euclidean(2);
    let mv: CausalMultiVector<f64> = CausalMultiVector::pseudoscalar(m);
    assert_eq!(mv.get(3), Some(&1.0)); // Index 3 is e1^e2 (11 binary)
    assert_eq!(mv.get(0), Some(&0.0));
}
