/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensorError;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum DataCleaningError {
    TensorError(CausalTensorError),
}

impl fmt::Display for DataCleaningError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataCleaningError::TensorError(e) => {
                write!(f, "DataCleaningError: Tensor Error: {}", e)
            }
        }
    }
}

impl std::error::Error for DataCleaningError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataCleaningError::TensorError(e) => Some(e),
        }
    }
}

impl From<CausalTensorError> for DataCleaningError {
    fn from(err: CausalTensorError) -> DataCleaningError {
        DataCleaningError::TensorError(err)
    }
}
