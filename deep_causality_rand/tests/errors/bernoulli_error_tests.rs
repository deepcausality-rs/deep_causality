/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::BernoulliDistributionError;
use std::error::Error;

#[test]
fn test_invalid_probability_display() {
    let error = BernoulliDistributionError::InvalidProbability;
    let expected = "p is outside [0, 1] in Bernoulli distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_invalid_probability_debug() {
    let error = BernoulliDistributionError::InvalidProbability;
    let expected = "InvalidProbability";
    assert_eq!(format!("{:?}", error), expected);
}

#[test]
fn test_invalid_probability_clone_copy_eq() {
    let error1 = BernoulliDistributionError::InvalidProbability;
    let error2 = error1; // Test Copy
    let error3 = error1.clone(); // Test Clone
    assert_eq!(error1, error2);
    assert_eq!(error1, error3);
}

#[test]
fn test_invalid_probability_error_trait() {
    let error = BernoulliDistributionError::InvalidProbability;
    let source = error.source();
    assert!(source.is_none());
}