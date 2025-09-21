/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensorError;
use std::fmt;

#[derive(Debug)]
pub enum AnalyzeError {
    EmptyResult,
    AnalysisFailed(String),
    TensorError(CausalTensorError),
}

impl fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnalyzeError::EmptyResult => write!(f, "The causal discovery result is empty."),
            AnalyzeError::AnalysisFailed(s) => write!(f, "Analysis failed: {}", s),
            AnalyzeError::TensorError(e) => write!(f, "Tensor error during analysis: {}", e),
        }
    }
}

impl std::error::Error for AnalyzeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalyzeError::TensorError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CausalTensorError> for AnalyzeError {
    fn from(err: CausalTensorError) -> AnalyzeError {
        AnalyzeError::TensorError(err)
    }
}
