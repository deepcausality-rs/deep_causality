/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::mrmr::MrmrError;
use deep_causality_tensor::CausalTensorError;
use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum FeatureSelectError {
    TooFewFeatures(usize, usize),
    MrmrError(MrmrError),
    TensorError(CausalTensorError),
}

impl fmt::Display for FeatureSelectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FeatureSelectError::TooFewFeatures(need, have) => write!(
                f,
                "Too few features available. Need at least {}, but found {}.",
                need, have
            ),
            FeatureSelectError::MrmrError(e) => write!(f, "mRMR algorithm error: {}", e),
            FeatureSelectError::TensorError(e) => write!(f, "Tensor error: {}", e),
        }
    }
}

impl std::error::Error for FeatureSelectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // MrmrError does not implement Error, so we can't return it as a source.
            FeatureSelectError::TensorError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<MrmrError> for FeatureSelectError {
    fn from(err: MrmrError) -> FeatureSelectError {
        FeatureSelectError::MrmrError(err)
    }
}
impl From<CausalTensorError> for FeatureSelectError {
    fn from(err: CausalTensorError) -> FeatureSelectError {
        FeatureSelectError::TensorError(err)
    }
}
