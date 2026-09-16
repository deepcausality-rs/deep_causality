/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::mrmr::MrmrError;

#[test]
fn test_invalid_input_error_display() {
    let error = MrmrError::InvalidInput("Input tensor is empty".to_string());
    assert_eq!(format!("{}", error), "Invalid input: Input tensor is empty");
}

#[test]
fn test_calculation_error_display() {
    let error = MrmrError::CalculationError("Division by zero in correlation".to_string());
    assert_eq!(
        format!("{}", error),
        "Calculation error: Division by zero in correlation"
    );
}

#[test]
fn test_not_enough_features_error_display() {
    let error = MrmrError::NotEnoughFeatures;
    assert_eq!(format!("{}", error), "Not enough features to select from.");
}

#[test]
fn test_sample_too_small_error_display() {
    let error = MrmrError::SampleTooSmall(3);
    assert_eq!(
        format!("{}", error),
        "Sample size is too small. At least 3 samples are required."
    );
}

#[test]
fn test_invalid_input_error_partial_eq() {
    let error1 = MrmrError::InvalidInput("Test error".to_string());
    let error2 = MrmrError::InvalidInput("Test error".to_string());
    let error3 = MrmrError::InvalidInput("Another error".to_string());
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_calculation_error_partial_eq() {
    let error1 = MrmrError::CalculationError("Test error".to_string());
    let error2 = MrmrError::CalculationError("Test error".to_string());
    let error3 = MrmrError::CalculationError("Another error".to_string());
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_uncertainty_error_display() {
    let error = MrmrError::UncertaintyError("Failed to sample".to_string());
    assert_eq!(format!("{}", error), "Uncertainty error: Failed to sample");
}

#[test]
fn test_uncertainty_error_partial_eq() {
    let error1 = MrmrError::UncertaintyError("Test error".to_string());
    let error2 = MrmrError::UncertaintyError("Test error".to_string());
    let error3 = MrmrError::UncertaintyError("Another error".to_string());
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_feature_score_error_display() {
    let error = MrmrError::FeatureScoreError("Division by zero".to_string());
    assert_eq!(
        format!("{}", error),
        "Feature score error: Division by zero"
    );
}

#[test]
fn test_error_boxes_as_std_error() {
    // A caller propagating an mRMR failure out of `main` boxes it behind `std::error::Error`.
    // That requires the trait impl, so this test fails to compile without it.
    fn propagate() -> Result<(), Box<dyn std::error::Error>> {
        Err(MrmrError::NotEnoughFeatures)?;
        Ok(())
    }

    let boxed = propagate().expect_err("the call returns the error it was given");
    assert_eq!(boxed.to_string(), "Not enough features to select from.");
}

#[test]
fn test_error_reports_no_source() {
    // Every variant carries its own message and wraps no underlying cause, so the source chain
    // ends at the error itself.
    let error = MrmrError::SampleTooSmall(3);
    let as_std: &dyn std::error::Error = &error;
    assert!(as_std.source().is_none());
}
