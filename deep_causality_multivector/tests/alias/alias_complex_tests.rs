/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{DixonAlgebra, Metric};
use deep_causality_num::Complex;

#[test]
fn test_dixon_algebra() {
    // Dixon State Space is Cl_C(6) -> 64 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 64];
    let d = DixonAlgebra::new_dixon_state_space(data);

    assert_eq!(d.metric().dimension(), 6);
    match d.metric() {
        Metric::NonEuclidean(6) => {}
        _ => panic!("Dixon State Space should be NonEuclidean(6)"),
    }
}

#[test]
fn test_complex_pauli() {
    // Complex Pauli Algebra is Cl_C(2) -> 4 dimensions (complex)
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 1.0),
        Complex::new(2.0, 0.0),
        Complex::new(0.0, 2.0),
    ];
    let cp = DixonAlgebra::new_complex_pauli(data.clone());

    assert_eq!(cp.metric().dimension(), 2);
    match cp.metric() {
        Metric::Euclidean(2) => {}
        _ => panic!("Complex Pauli Algebra should be Euclidean(2)"),
    }
    assert_eq!(cp.data().len(), 4);
    assert_eq!(cp.data(), &data);
}

#[test]
fn test_octonion_operator() {
    // Octonion Operator is Cl_C(6) -> 64 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 64];
    let oo = DixonAlgebra::new_octonion_operator(data);

    assert_eq!(oo.metric().dimension(), 6);
    match oo.metric() {
        Metric::NonEuclidean(6) => {}
        _ => panic!("Octonion Operator should be NonEuclidean(6)"),
    }
    assert_eq!(oo.data().len(), 64);
}

#[test]
fn test_quaternion_operator() {
    // Quaternion Operator is Cl_C(4) -> 16 dimensions (complex)
    let dim = 4;
    let size = 1 << dim;
    // Assuming DixonAlgebra is an alias for ComplexMultiVector
    let data = vec![Complex::new(0.0, 0.0); size];
    let qo = DixonAlgebra::new_quaternion_operator(data);

    // 1. Check Dimension
    assert_eq!(qo.metric().dimension(), dim);

    // 2. Check Metric Signature
    match qo.metric() {
        Metric::NonEuclidean(4) => {}
        _ => panic!("Quaternion Operator should be NonEuclidean(4)"),
    }

    // 3. Check Data Length (2^4 = 16)
    assert_eq!(qo.data().len(), size);
}

#[test]
fn test_gut_algebra() {
    // GUT Algebra is Cl_C(10) -> 1024 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 1024];
    let gut = DixonAlgebra::new_gut_algebra(data);

    assert_eq!(gut.metric().dimension(), 10);
    match gut.metric() {
        Metric::NonEuclidean(10) => {}
        _ => panic!("GUT Algebra should be NonEuclidean(10)"),
    }
    assert_eq!(gut.data().len(), 1024);
}
