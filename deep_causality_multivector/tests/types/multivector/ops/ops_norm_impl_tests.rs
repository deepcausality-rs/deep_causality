/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::MultiVectorL2Norm;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Complex;

#[test]
fn test_real_norm_l2() {
    let data = vec![0.0, 3.0, 4.0, 0.0];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap();
    let norm = mv.norm_l2();
    assert_eq!(norm, 5.0);
}

#[test]
fn test_real_normalize_l2() {
    let data = vec![0.0, 3.0, 4.0, 0.0];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap();
    let normalized_mv = mv.normalize_l2();

    let expected_data: Vec<f64> = vec![0.0, 0.6, 0.8, 0.0];

    // Compare elements with tolerance instead of strict equality
    let tolerance = 1e-9;
    for (val, expected) in normalized_mv.data().iter().zip(expected_data.iter()) {
        assert!(
            (val - expected).abs() < tolerance,
            "Mismatch: {} != {}",
            val,
            expected
        );
    }

    // The norm of the normalized vector should be 1.0
    let norm: f64 = normalized_mv.norm_l2();
    assert!((norm - 1.0f64).abs() < tolerance);
}

#[test]
fn test_real_normalize_l2_zero_vector() {
    let data = vec![0.0, 0.0, 0.0, 0.0];
    let mv = CausalMultiVector::new(data.clone(), Metric::Euclidean(2)).unwrap();
    let normalized_mv = mv.normalize_l2();
    assert_eq!(normalized_mv.data(), &data);
    assert_eq!(normalized_mv.norm_l2(), 0.0);
}

#[test]
fn test_complex_norm_l2() {
    let data = vec![
        Complex::new(3.0, 4.0), // |c|^2 = 25
        Complex::new(0.0, 0.0),
    ];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(1)).unwrap();
    let norm = mv.norm_l2();
    assert_eq!(norm, 5.0);
}

#[test]
fn test_complex_normalize_l2() {
    let data = vec![
        Complex::new(3.0, 4.0), // norm is 5.0
        Complex::new(0.0, 0.0),
        Complex::new(0.0, -3.0), // norm is 3.0
        Complex::new(4.0, 0.0),  // norm is 4.0
    ];
    // Total norm_sq = 25 + 0 + 9 + 16 = 50. Norm = sqrt(50)
    let mv = CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap();
    let normalized_mv = mv.normalize_l2();

    let norm: f64 = normalized_mv.norm_l2();
    assert!((norm - 1.0).abs() < 1e-9, "Norm is {}", norm);

    let norm_sqrt_50 = 50.0f64.sqrt();
    let expected_data = [
        Complex::new(3.0 / norm_sqrt_50, 4.0 / norm_sqrt_50),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, -3.0 / norm_sqrt_50),
        Complex::new(4.0 / norm_sqrt_50, 0.0),
    ];

    for (a, b) in normalized_mv.data().iter().zip(expected_data.iter()) {
        assert!((a.re - b.re).abs() < 1e-9);
        assert!((a.im - b.im).abs() < 1e-9);
    }
}

#[test]
fn test_complex_normalize_l2_zero_vector() {
    let data = vec![
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv = CausalMultiVector::new(data.clone(), Metric::Euclidean(2)).unwrap();
    let normalized_mv = mv.normalize_l2();
    assert_eq!(normalized_mv.data(), &data);
    assert_eq!(normalized_mv.norm_l2(), 0.0);
}
