/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{DixonAlgebra, Metric};
use deep_causality_num::Complex;

#[test]
fn test_dixon_algebra() {
    // Dixon Algebra is Cl_C(6) -> 64 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 64];
    let d = DixonAlgebra::new_dixon_algebra(data);

    assert_eq!(d.metric.dimension(), 6);
    match d.metric {
        Metric::Euclidean(6) => {}
        _ => panic!("Dixon Algebra should be Euclidean(6)"),
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

    assert_eq!(cp.metric.dimension(), 2);
    match cp.metric {
        Metric::Euclidean(2) => {}
        _ => panic!("Complex Pauli Algebra should be Euclidean(2)"),
    }
    assert_eq!(cp.data.len(), 4);
    assert_eq!(cp.data, data);
}

#[test]
fn test_octonion_operator() {
    // Octonion Operator is Cl_C(6) -> 64 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 64];
    let oo = DixonAlgebra::new_octonion_operator(data);

    assert_eq!(oo.metric.dimension(), 6);
    match oo.metric {
        Metric::Euclidean(6) => {}
        _ => panic!("Octonion Operator should be Euclidean(6)"),
    }
    assert_eq!(oo.data.len(), 64);
}

#[test]
fn test_gut_algebra() {
    // GUT Algebra is Cl_C(10) -> 1024 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 1024];
    let gut = DixonAlgebra::new_gut_algebra(data);

    assert_eq!(gut.metric.dimension(), 10);
    match gut.metric {
        Metric::Euclidean(10) => {}
        _ => panic!("GUT Algebra should be Euclidean(10)"),
    }
    assert_eq!(gut.data.len(), 1024);
}
