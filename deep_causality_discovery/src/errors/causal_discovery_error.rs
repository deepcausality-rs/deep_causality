/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensorError;
use std::fmt;

#[derive(Debug)]
pub enum CausalDiscoveryError {
    TensorError(CausalTensorError),
}

impl fmt::Display for CausalDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CausalDiscoveryError::TensorError(e) => write!(f, "Tensor error during SURD: {}", e),
        }
    }
}

impl std::error::Error for CausalDiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CausalDiscoveryError::TensorError(e) => Some(e),
        }
    }
}

impl From<CausalTensorError> for CausalDiscoveryError {
    fn from(err: CausalTensorError) -> CausalDiscoveryError {
        CausalDiscoveryError::TensorError(err)
    }
}
