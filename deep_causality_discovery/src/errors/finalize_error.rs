/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum FinalizeError {
    FormattingError(String),
}

impl fmt::Display for FinalizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FinalizeError::FormattingError(s) => write!(f, "Formatting error: {}", s),
        }
    }
}

impl std::error::Error for FinalizeError {}
