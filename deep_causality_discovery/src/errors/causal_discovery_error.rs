/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::BrcdError;
use deep_causality_tensor::CausalTensorError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CausalDiscoveryError {
    TensorError(CausalTensorError),
    Brcd(BrcdError),
}

impl fmt::Display for CausalDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CausalDiscoveryError::TensorError(e) => {
                write!(f, "Tensor error during discovery: {}", e)
            }
            CausalDiscoveryError::Brcd(e) => write!(f, "BRCD error during discovery: {}", e),
        }
    }
}

impl std::error::Error for CausalDiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CausalDiscoveryError::TensorError(e) => Some(e),
            CausalDiscoveryError::Brcd(e) => Some(e),
        }
    }
}

impl From<CausalTensorError> for CausalDiscoveryError {
    fn from(err: CausalTensorError) -> CausalDiscoveryError {
        CausalDiscoveryError::TensorError(err)
    }
}

impl From<BrcdError> for CausalDiscoveryError {
    fn from(err: BrcdError) -> CausalDiscoveryError {
        CausalDiscoveryError::Brcd(err)
    }
}
