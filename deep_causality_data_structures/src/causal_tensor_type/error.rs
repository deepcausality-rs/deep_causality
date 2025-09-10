/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::error::Error;

/// Errors that can occur during tensor operations.
#[derive(Debug, PartialEq)]
pub enum CausalTensorError {
    ShapeMismatch,
    DimensionMismatch,
    AxisOutOfBounds,
    EmptyTensor,
    InvalidOperation,
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
        }
    }
}
