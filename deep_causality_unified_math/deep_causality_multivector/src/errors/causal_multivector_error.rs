/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Metric;
use core::fmt;

/// The main error type for CausalMultiVector operations.
///
/// This struct wraps `CausalMultiVectorErrorInner` to provide a stable API while allowing internal evolution.
#[derive(Debug, PartialEq)]
pub struct CausalMultiVectorError {
    inner: CausalMultiVectorErrorInner,
}

/// Internal enum for specific error variants.
#[derive(Debug, Clone, PartialEq)]
pub enum CausalMultiVectorErrorInner {
    /// Error when operations are performed on vectors of different dimensions.
    DimensionMismatch { expected: usize, found: usize },
    /// Error when the data length provided does not match $2^N$.
    DataLengthMismatch { expected: usize, found: usize },
    /// Error when an operation requires a non-zero magnitude (e.g., inverse).
    ZeroMagnitude,
    /// Error when operations are performed on vectors with different metrics.
    MetricMismatch { left: Metric, right: Metric },
    /// Error when an operation must extract a value from a multivector that stores none.
    ///
    /// Raised by the partial `Adjunction` operations, `counit` and `right_adjunct`, which return a
    /// bare `B` taken out of the container. An empty multivector has no `B` to give.
    EmptyMultiVector,
}

impl core::error::Error for CausalMultiVectorError {}

impl CausalMultiVectorError {
    /// Creates an EmptyMultiVector error, for an extraction from a multivector storing no value.
    pub fn empty_multivector() -> Self {
        Self {
            inner: CausalMultiVectorErrorInner::EmptyMultiVector,
        }
    }

    /// Creates a DimensionMismatch error.
    pub fn dimension_mismatch(expected: usize, found: usize) -> Self {
        Self {
            inner: CausalMultiVectorErrorInner::DimensionMismatch { expected, found },
        }
    }

    /// Creates a DataLengthMismatch error.
    pub fn data_length_mismatch(expected: usize, found: usize) -> Self {
        Self {
            inner: CausalMultiVectorErrorInner::DataLengthMismatch { expected, found },
        }
    }

    /// Creates a ZeroMagnitude error.
    pub fn zero_magnitude() -> Self {
        Self {
            inner: CausalMultiVectorErrorInner::ZeroMagnitude,
        }
    }

    /// Creates a MetricMismatch error.
    pub fn metric_mismatch(left: Metric, right: Metric) -> Self {
        Self {
            inner: CausalMultiVectorErrorInner::MetricMismatch { left, right },
        }
    }
}

impl fmt::Display for CausalMultiVectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            CausalMultiVectorErrorInner::DimensionMismatch { expected, found } => {
                write!(
                    f,
                    "Dimension mismatch: expected {}, found {}",
                    expected, found
                )
            }
            CausalMultiVectorErrorInner::DataLengthMismatch { expected, found } => {
                write!(
                    f,
                    "Data length mismatch: expected {}, found {}",
                    expected, found
                )
            }
            CausalMultiVectorErrorInner::ZeroMagnitude => {
                write!(
                    f,
                    "Operation requires non-zero magnitude (e.g., inverse of zero)"
                )
            }
            CausalMultiVectorErrorInner::MetricMismatch { left, right } => {
                write!(
                    f,
                    "Metric mismatch between operands: {:?} vs {:?}",
                    left, right
                )
            }
            CausalMultiVectorErrorInner::EmptyMultiVector => {
                write!(
                    f,
                    "Operation requires a multivector that stores at least one value"
                )
            }
        }
    }
}
