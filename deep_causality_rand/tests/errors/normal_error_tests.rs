/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::NormalDistributionError;
use std::error::Error;

#[test]
fn test_mean_too_small_display() {
    let error = NormalDistributionError::MeanTooSmall;
    let expected = "mean < 0 or NaN in log-normal distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_bad_variance_display() {
    let error = NormalDistributionError::BadVariance;
    let expected = "variation parameter is non-finite in (log)normal distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_normal_distribution_error_debug() {
    let error_mean = NormalDistributionError::MeanTooSmall;
    assert_eq!(format!("{:?}", error_mean), "MeanTooSmall");

    let error_variance = NormalDistributionError::BadVariance;
    assert_eq!(format!("{:?}", error_variance), "BadVariance");
}

#[test]
fn test_normal_distribution_error_clone_copy_eq() {
    let error1 = NormalDistributionError::MeanTooSmall;
    let error2 = error1; // Test Copy
    let error3 = error1.clone(); // Test Clone
    assert_eq!(error1, error2);
    assert_eq!(error1, error3);

    let error4 = NormalDistributionError::BadVariance;
    let error5 = error4; // Test Copy
    let error6 = error4.clone(); // Test Clone
    assert_eq!(error4, error5);
    assert_eq!(error4, error6);

    assert_ne!(error1, error4);
}

#[test]
fn test_normal_distribution_error_trait() {
    let error_mean = NormalDistributionError::MeanTooSmall;
    assert!(error_mean.source().is_none());

    let error_variance = NormalDistributionError::BadVariance;
    assert!(error_variance.source().is_none());
}