/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum PreprocessError {
    InvalidColumnIdentifier(String),
    BinningError(String),
    ConfigError(String),
    ImputeError(String),
}

impl std::error::Error for PreprocessError {}

impl fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreprocessError::InvalidColumnIdentifier(s) => {
                write!(f, "Invalid column identifier: {}", s)
            }
            PreprocessError::BinningError(s) => write!(f, "Binning error: {}", s),
            PreprocessError::ConfigError(s) => {
                write!(f, "Invalid preprocessing configuration: {}", s)
            }
            PreprocessError::ImputeError(s) => write!(f, "Imputation error: {}", s),
        }
    }
}
