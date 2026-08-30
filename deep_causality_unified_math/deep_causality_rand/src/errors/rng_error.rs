/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::string::String;
use alloc::string::ToString;

use crate::UniformDistributionError;
use core::error::Error;

#[derive(Debug, PartialEq)]
pub enum RngError {
    OsRandomGenerator(String),
    InvalidRange(String),
    UnsupportedDimension(String),
}

impl Error for RngError {}

impl core::fmt::Display for RngError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RngError::OsRandomGenerator(e) => write!(f, "OS random generator error: {}", e),
            RngError::InvalidRange(e) => write!(f, "Invalid range: {}", e),
            RngError::UnsupportedDimension(e) => write!(f, "Unsupported dimension: {}", e),
        }
    }
}

impl From<UniformDistributionError> for RngError {
    fn from(e: UniformDistributionError) -> Self {
        RngError::InvalidRange(e.to_string())
    }
}
