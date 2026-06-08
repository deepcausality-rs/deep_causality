/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for the SURD analysis phase of the CDL pipeline.
///
/// Holds the thresholds that decide what counts as a "strong" synergistic,
/// unique, or redundant influence when interpreting a `SurdResult`.
#[derive(Debug, Clone)]
pub struct SurdAnalyzeConfig {
    synergy_threshold: f64,
    unique_threshold: f64,
    redundancy_threshold: f64,
}

impl SurdAnalyzeConfig {
    /// Creates a new `SurdAnalyzeConfig` with the specified thresholds.
    pub fn new(synergy_threshold: f64, unique_threshold: f64, redundancy_threshold: f64) -> Self {
        Self {
            synergy_threshold,
            unique_threshold,
            redundancy_threshold,
        }
    }

    /// The minimum value for a synergistic influence to be considered significant.
    pub fn synergy_threshold(&self) -> f64 {
        self.synergy_threshold
    }

    /// The minimum value for a unique influence to be considered significant.
    pub fn unique_threshold(&self) -> f64 {
        self.unique_threshold
    }

    /// The minimum value for a redundant influence to be considered significant.
    pub fn redundancy_threshold(&self) -> f64 {
        self.redundancy_threshold
    }
}

impl Default for SurdAnalyzeConfig {
    /// The default interpretive thresholds used when no config is supplied.
    fn default() -> Self {
        Self::new(0.01, 0.01, 0.01)
    }
}

impl fmt::Display for SurdAnalyzeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SurdAnalyzeConfig(synergy: {}, unique: {}, redundancy: {})",
            self.synergy_threshold, self.unique_threshold, self.redundancy_threshold
        )
    }
}
