/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone)]
pub struct AnalyzeConfig {
    synergy_threshold: f64,
    unique_threshold: f64,
    redundancy_threshold: f64,
}

impl AnalyzeConfig {
    pub fn new(synergy_threshold: f64, unique_threshold: f64, redundancy_threshold: f64) -> Self {
        Self {
            synergy_threshold,
            unique_threshold,
            redundancy_threshold,
        }
    }

    pub fn synergy_threshold(&self) -> f64 {
        self.synergy_threshold
    }

    pub fn unique_threshold(&self) -> f64 {
        self.unique_threshold
    }

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
