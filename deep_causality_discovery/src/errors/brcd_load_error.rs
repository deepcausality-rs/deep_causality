/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::cpdag_error::CpdagError;
use crate::errors::data_loading_error::DataLoadingError;
use std::fmt;

/// Failure cases for `BrcdDataLoader` building a [`crate::BrcdInput`].
#[derive(Debug, Clone, PartialEq)]
pub enum BrcdLoadError {
    /// Loading one of the two datasets failed.
    DataLoading(DataLoadingError),
    /// Loading or parsing the CPDAG file failed.
    Cpdag(CpdagError),
    /// The datasets/graph disagree on shape: a dataset is not 2-D, the two
    /// datasets have different variable counts, or the CPDAG's vertex count does
    /// not equal the variable count.
    DimensionMismatch(String),
    /// Casting or constructing a tensor at the pipeline precision failed.
    Tensor(String),
    /// Learning the CPDAG via BOSS, or persisting it to the keyed cache
    /// (CSV or key sidecar), failed.
    Learning(String),
}

impl fmt::Display for BrcdLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrcdLoadError::DataLoading(e) => write!(f, "BRCD data loading failed: {}", e),
            BrcdLoadError::Cpdag(e) => write!(f, "BRCD CPDAG loading failed: {}", e),
            BrcdLoadError::DimensionMismatch(e) => {
                write!(f, "BRCD input dimension mismatch: {}", e)
            }
            BrcdLoadError::Tensor(e) => write!(f, "BRCD tensor construction failed: {}", e),
            BrcdLoadError::Learning(e) => write!(f, "BRCD CPDAG learning/caching failed: {}", e),
        }
    }
}

impl std::error::Error for BrcdLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BrcdLoadError::DataLoading(e) => Some(e),
            BrcdLoadError::Cpdag(e) => Some(e),
            BrcdLoadError::DimensionMismatch(_)
            | BrcdLoadError::Tensor(_)
            | BrcdLoadError::Learning(_) => None,
        }
    }
}

impl From<DataLoadingError> for BrcdLoadError {
    fn from(err: DataLoadingError) -> BrcdLoadError {
        BrcdLoadError::DataLoading(err)
    }
}

impl From<CpdagError> for BrcdLoadError {
    fn from(err: CpdagError) -> BrcdLoadError {
        BrcdLoadError::Cpdag(err)
    }
}
