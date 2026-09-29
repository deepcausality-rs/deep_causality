/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::RngError;
use deep_causality_rand::UniformDistributionError;
use std::error::Error;

#[test]
fn test_os_random_generator_display() {
    let error = RngError::OsRandomGenerator;
    assert_eq!(format!("{}", error), "OS random generator error");
}

#[test]
fn test_invalid_range_display() {
    let error = RngError::InvalidRange(UniformDistributionError::EmptyRange);
    let expected = "Invalid range: Empty range in uniform distribution";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_unsupported_dimension_display() {
    let error = RngError::UnsupportedDimension {
        dimension: 0,
        max: 16,
    };
    let expected = "Unsupported dimension: dimension 0 outside 1..=16";
    assert_eq!(format!("{}", error), expected);
}

#[test]
fn test_rng_error_debug() {
    assert_eq!(
        format!("{:?}", RngError::OsRandomGenerator),
        "OsRandomGenerator"
    );
    assert_eq!(
        format!(
            "{:?}",
            RngError::InvalidRange(UniformDistributionError::NonFinite)
        ),
        "InvalidRange(NonFinite)"
    );
    assert_eq!(
        format!(
            "{:?}",
            RngError::UnsupportedDimension {
                dimension: 17,
                max: 16
            }
        ),
        "UnsupportedDimension { dimension: 17, max: 16 }"
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
    assert_eq!(
        rng_error,
        RngError::InvalidRange(UniformDistributionError::InvalidRange)
    );
}

#[test]
fn test_rng_error_from_keeps_every_uniform_variant() {
    for e in [
        UniformDistributionError::NonFinite,
        UniformDistributionError::InvalidRange,
        UniformDistributionError::EmptyRange,
    ] {
        assert_eq!(RngError::from(e), RngError::InvalidRange(e));
    }
}

#[test]
fn test_rng_error_trait() {
    assert!(RngError::OsRandomGenerator.source().is_none());
    assert!(
        RngError::InvalidRange(UniformDistributionError::InvalidRange)
            .source()
            .is_none()
    );
}
