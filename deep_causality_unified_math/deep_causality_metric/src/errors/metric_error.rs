/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt;

/// Errors that can occur during metric operations.
///
/// Each variant carries a fixed message, so the type needs no allocator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    /// Sign convention mismatch (e.g., expected East Coast, got West Coast)
    SignConventionMismatch(&'static str),
    /// Invalid dimension (e.g., zero or exceeds bitmask capacity)
    InvalidDimension(&'static str),
    /// Metric validation failed
    ValidationFailed(&'static str),
    /// Conversion not possible
    ConversionError(&'static str),
}

impl MetricError {
    /// Creates a SignConventionMismatch error.
    pub fn sign_convention_mismatch(msg: &'static str) -> Self {
        Self::SignConventionMismatch(msg)
    }

    /// Creates an InvalidDimension error.
    pub fn invalid_dimension(msg: &'static str) -> Self {
        Self::InvalidDimension(msg)
    }

    /// Creates a ValidationFailed error.
    pub fn validation_failed(msg: &'static str) -> Self {
        Self::ValidationFailed(msg)
    }

    /// Creates a ConversionError.
    pub fn conversion_error(msg: &'static str) -> Self {
        Self::ConversionError(msg)
    }
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricError::SignConventionMismatch(msg) => {
                write!(f, "Sign convention mismatch: {}", msg)
            }
            MetricError::InvalidDimension(msg) => {
                write!(f, "Invalid dimension: {}", msg)
            }
            MetricError::ValidationFailed(msg) => {
                write!(f, "Metric validation failed: {}", msg)
            }
            MetricError::ConversionError(msg) => {
                write!(f, "Conversion error: {}", msg)
            }
        }
    }
}

impl core::error::Error for MetricError {}
