/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::UniformDistributionError;
use core::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RngError {
    /// The operating system's random source failed.
    OsRandomGenerator,
    /// The range is empty, reversed or not finite.
    InvalidRange(UniformDistributionError),
    /// A Sobol dimension outside `1..=max`.
    UnsupportedDimension { dimension: usize, max: usize },
}

impl Error for RngError {}

impl core::fmt::Display for RngError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RngError::OsRandomGenerator => write!(f, "OS random generator error"),
            RngError::InvalidRange(e) => write!(f, "Invalid range: {}", e),
            RngError::UnsupportedDimension { dimension, max } => write!(
                f,
                "Unsupported dimension: dimension {} outside 1..={}",
                dimension, max
            ),
        }
    }
}

impl From<UniformDistributionError> for RngError {
    fn from(e: UniformDistributionError) -> Self {
        RngError::InvalidRange(e)
    }
}
