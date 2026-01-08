/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for the analysis phase of the CDL pipeline.
///
/// This struct holds thresholds that determine what is considered a "strong"
/// causal influence when interpreting the results of a discovery algorithm.
#[derive(Debug, Clone)]
pub struct AnalyzeConfig {
    synergy_threshold: f64,
    unique_threshold: f64,
    redundancy_threshold: f64,
}

impl AnalyzeConfig {
    /// Creates a new `AnalyzeConfig` with the specified thresholds.
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

impl fmt::Display for AnalyzeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AnalyzeConfig(synergy: {}, unique: {}, redundancy: {})",
            self.synergy_threshold, self.unique_threshold, self.redundancy_threshold
        )
    }
}
