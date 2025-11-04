/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
    /// Encapsulates errors specific to EinSum AST validation and execution.
    EinSumError(EinSumValidationError),
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
            CausalTensorError::EinSumError(e) => {
                write!(f, "CausalTensorError: EinSumError: {}", e)
            }
        }
    }
}
