/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::UniformDistributionError;
use std::error::Error;

#[test]
fn test_non_finite_display() {
    let error = UniformDistributionError::NonFinite;
    let expected = "Non-finite range in uniform distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_invalid_range_display() {
    let error = UniformDistributionError::InvalidRange;
    let expected = "Invalid range: low must be less than high";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_empty_range_display() {
    let error = UniformDistributionError::EmptyRange;
    let expected = "Empty range in uniform distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_uniform_distribution_error_debug() {
    let error_non_finite = UniformDistributionError::NonFinite;
    assert_eq!(format!("{:?}", error_non_finite), "NonFinite");

    let error_invalid_range = UniformDistributionError::InvalidRange;
    assert_eq!(format!("{:?}", error_invalid_range), "InvalidRange");

    let error_empty_range = UniformDistributionError::EmptyRange;
    assert_eq!(format!("{:?}", error_empty_range), "EmptyRange");
}

#[test]
fn test_uniform_distribution_error_clone_copy_eq() {
    let error1 = UniformDistributionError::NonFinite;
    let error2 = error1; // Test Copy
    let error3 = error1.clone(); // Test Clone
    assert_eq!(error1, error2);
    assert_eq!(error1, error3);

    let error4 = UniformDistributionError::InvalidRange;
    assert_ne!(error1, error4);
}

#[test]
fn test_uniform_distribution_error_trait() {
    let error = UniformDistributionError::NonFinite;
    assert!(error.source().is_none());
}