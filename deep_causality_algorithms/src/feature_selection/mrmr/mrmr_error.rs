/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Errors that can occur during mRMR feature selection.
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum MrmrError {
    /// Indicates that the input data (e.g., CausalTensor dimensions, number of features requested)
    /// is invalid or does not meet the algorithm's requirements.
    InvalidInput(String),
    /// Signifies a numerical issue encountered during statistical calculations (e.g., division by zero,
    /// or other floating-point anomalies that prevent a meaningful result).
    CalculationError(String),
    /// Occurs when the requested number of features to select is greater than the total number of
    /// available features in the input tensor (excluding the target variable).
    NotEnoughFeatures,
    /// Indicates that the sample size (number of rows in the CausalTensor) is too small for the
    /// statistical calculations (e.g., Pearson correlation requires at least 2 samples, F-statistic requires at least 3).
    SampleTooSmall(usize),
    /// For errors originating from `MaybeUncertain` operations.
    UncertaintyError(String),
}

impl fmt::Display for MrmrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MrmrError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            MrmrError::CalculationError(s) => write!(f, "Calculation error: {}", s),
            MrmrError::NotEnoughFeatures => write!(f, "Not enough features to select from."),
            MrmrError::SampleTooSmall(n) => write!(
                f,
                "Sample size is too small. At least {} samples are required.",
                n
            ),
            MrmrError::UncertaintyError(s) => write!(f, "Uncertainty error: {}", s),
        }
    }
}
