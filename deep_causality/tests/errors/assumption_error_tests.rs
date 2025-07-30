/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::AssumptionError;

#[test]
fn test_no_assumptions_defined_error() {
    let error = AssumptionError::NoAssumptionsDefined;
    assert_eq!(error.to_string(), "Model has no assumptions to verify");
}

#[test]
fn test_no_data_error() {
    let error = AssumptionError::NoDataToTestDefined;
    assert_eq!(error.to_string(), "No Data to test provided");
}

#[test]
fn test_evaluation_failed_error() {
    let error = AssumptionError::EvaluationFailed("Test Error".to_string());
    assert_eq!(
        error.to_string(),
        "Failed to evaluate assumption: Test Error"
    );
}
