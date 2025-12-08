/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::UncertainParameter;

#[test]
fn test_new() {
    let threshold = 0.5;
    let confidence = 0.9;
    let epsilon = 0.01;
    let max_samples = 500;

    let params = UncertainParameter::new(threshold, confidence, epsilon, max_samples);

    assert_eq!(params.threshold(), threshold);
    assert_eq!(params.confidence(), confidence);
    assert_eq!(params.epsilon(), epsilon);
    assert_eq!(params.max_samples(), max_samples);
}

#[test]
fn test_default() {
    let params = UncertainParameter::default();

    assert_eq!(params.threshold(), 0.8);
    assert_eq!(params.confidence(), 0.95);
    assert_eq!(params.epsilon(), 0.05);
    assert_eq!(params.max_samples(), 1000);
}

#[test]
fn test_getters() {
    let params = UncertainParameter::new(0.6, 0.99, 0.02, 200);

    assert_eq!(params.threshold(), 0.6);
    assert_eq!(params.confidence(), 0.99);
    assert_eq!(params.epsilon(), 0.02);
    assert_eq!(params.max_samples(), 200);
}

#[test]
fn test_clone_and_partial_eq() {
    let params1 = UncertainParameter::new(0.7, 0.8, 0.03, 300);
    let params2 = params1.clone();

    assert_eq!(params1, params2);

    let params3 = UncertainParameter::default();
    assert_ne!(params1, params3);
}

#[test]
fn test_partial_ord() {
    let params1 = UncertainParameter::new(0.5, 0.9, 0.05, 1000);
    let params2 = UncertainParameter::new(0.6, 0.9, 0.05, 1000);
    let params3 = UncertainParameter::new(0.5, 0.95, 0.05, 1000);
    let params4 = UncertainParameter::new(0.5, 0.9, 0.06, 1000);
    let params5 = UncertainParameter::new(0.5, 0.9, 0.05, 1001);

    assert!(params1 < params2); // threshold
    assert!(params1 < params3); // confidence
    assert!(params1 < params4); // epsilon
    assert!(params1 < params5); // max_samples

    assert!(params2 > params1);
}

#[test]
fn test_debug() {
    let params = UncertainParameter::new(0.5, 0.9, 0.01, 500);
    let debug_str = format!("{:?}", params);

    assert!(debug_str.contains("UncertainParameter"));
    assert!(debug_str.contains("threshold: 0.5"));
    assert!(debug_str.contains("confidence: 0.9"));
    assert!(debug_str.contains("epsilon: 0.01"));
    assert!(debug_str.contains("max_samples: 500"));
}

#[test]
fn test_display() {
    let params = UncertainParameter::new(0.5, 0.9, 0.01, 500);

    let expected_output =
        "UncertainParameter { threshold: 0.5, confidence: 0.9, epsilon: 0.01, max_samples: 500 }";
    assert_eq!(format!("{}", params), expected_output);
}
