/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
