/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for the BRCD analysis phase of the CDL pipeline.
///
/// Holds how many top-ranked candidate root-cause sets to render from a
/// `BrcdResult` posterior ranking.
#[derive(Debug, Clone)]
pub struct BrcdAnalyzeConfig {
    top_k: usize,
}

impl BrcdAnalyzeConfig {
    /// Creates a new `BrcdAnalyzeConfig` reporting the `top_k` best candidates.
    pub fn new(top_k: usize) -> Self {
        Self { top_k }
    }

    /// The number of top-ranked candidate root-cause sets to report.
    pub fn top_k(&self) -> usize {
        self.top_k
    }
}

impl Default for BrcdAnalyzeConfig {
    /// Reports the top 5 candidates when no config is supplied.
    fn default() -> Self {
        Self::new(5)
    }
}

impl fmt::Display for BrcdAnalyzeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BrcdAnalyzeConfig(top_k: {})", self.top_k)
    }
}
