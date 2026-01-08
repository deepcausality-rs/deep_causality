/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EinSumValidationError;
use std::error::Error;

/// Errors that can occur during tensor operations.
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum CausalTensorError {
    ShapeMismatch,
    DimensionMismatch,
    DivisionByZero,
    AxisOutOfBounds,
    EmptyTensor,
    InvalidOperation,
    UnorderableValue,
    InvalidParameter(String),
    SingularMatrix,   // Added for matrix inversion errors
    IndexOutOfBounds, // Added for out-of-bounds access errors
    /// Encapsulates errors specific to EinSum AST validation and execution.
    EinSumError(EinSumValidationError),
    /// MLX array conversion failed.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    MlxConversionFailed,
    /// MLX evaluation failed.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    MlxEvalFailed,
    /// MLX operation failed with a message.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    MlxOperationFailed(String),
    /// Operation not implemented for this backend.
    NotImplemented(String),
}

impl Error for CausalTensorError {}

impl std::fmt::Display for CausalTensorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CausalTensorError::ShapeMismatch => {
                write!(f, "CausalTensorError: Shape mismatch error")
            }
            CausalTensorError::DimensionMismatch => {
                write!(f, "CausalTensorError: Dimension mismatch error")
            }
            CausalTensorError::AxisOutOfBounds => {
                write!(f, "CausalTensorError: Axis out of bounds error")
            }
            CausalTensorError::EmptyTensor => write!(f, "CausalTensorError: Empty tensor error"),
            CausalTensorError::InvalidOperation => {
                write!(f, "CausalTensorError: Invalid operation error")
            }
            CausalTensorError::UnorderableValue => {
                write!(f, "CausalTensorError: Unorderable value encountered")
            }
            CausalTensorError::InvalidParameter(s) => {
                write!(f, "CausalTensorError: Invalid parameter: {}", s)
            }
            CausalTensorError::DivisionByZero => {
                write!(f, "CausalTensorError: Division by zero error")
            }
            CausalTensorError::SingularMatrix => {
                write!(
                    f,
                    "CausalTensorError: Singular matrix error - inverse does not exist"
                )
            }
            CausalTensorError::IndexOutOfBounds => {
                write!(f, "CausalTensorError: Index out of bounds error")
            }
            CausalTensorError::EinSumError(e) => {
                write!(f, "CausalTensorError: EinSumError: {}", e)
            }
            #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
            CausalTensorError::MlxConversionFailed => {
                write!(f, "CausalTensorError: MLX array conversion failed")
            }
            #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
            CausalTensorError::MlxEvalFailed => {
                write!(f, "CausalTensorError: MLX evaluation failed")
            }
            #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
            CausalTensorError::MlxOperationFailed(msg) => {
                write!(f, "CausalTensorError: MLX operation failed: {}", msg)
            }
            CausalTensorError::NotImplemented(msg) => {
                write!(f, "CausalTensorError: Not implemented: {}", msg)
            }
        }
    }
}
