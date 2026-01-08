/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::RngError;
use deep_causality_rand::UniformDistributionError;
use std::error::Error;

#[test]
fn test_os_random_generator_display() {
    let error = RngError::OsRandomGenerator("test error".to_string());
    let expected = "OS random generator error: test error";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_invalid_range_display() {
    let error = RngError::InvalidRange("test range".to_string());
    let expected = "Invalid range: test range";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_rng_error_debug() {
    let error_os = RngError::OsRandomGenerator("debug error".to_string());
    assert_eq!(
        format!("{:?}", error_os),
        r#"OsRandomGenerator("debug error")"#
    );

    let error_range = RngError::InvalidRange("debug range".to_string());
    assert_eq!(
        format!("{:?}", error_range),
        r#"InvalidRange("debug range")"#
    );
}

#[test]
fn test_rng_error_from_uniform_distribution_error() {
    let uniform_error = UniformDistributionError::InvalidRange;
    let rng_error: RngError = uniform_error.into();

    let expected_msg = "Invalid range: low must be less than high";
    assert_eq!(
        format!("{}", rng_error),
        format!("Invalid range: {}", expected_msg)
    );

    if let RngError::InvalidRange(msg) = rng_error {
        assert_eq!(msg, expected_msg);
    } else {
        panic!("Conversion from UniformDistributionError failed");
    }
}

#[test]
fn test_rng_error_trait() {
    let error_os = RngError::OsRandomGenerator("source error".to_string());
    assert!(error_os.source().is_none());

    let error_range = RngError::InvalidRange("source range".to_string());
    assert!(error_range.source().is_none());
}
