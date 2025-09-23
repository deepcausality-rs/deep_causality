/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use std::fmt;

/// Configuration for the SURD (Synergistic, Unique, Redundant Decomposition) algorithm.
#[derive(Debug, Clone)]
pub struct SurdConfig {
    max_order: MaxOrder,
    target_col: usize,
}

impl SurdConfig {
    /// Creates a new `SurdConfig`.
    pub fn new(max_order: MaxOrder, target_col: usize) -> Self {
        Self {
            max_order,
            target_col,
        }
    }

    /// The maximum order of causal interactions to compute (e.g., pairwise, three-way).
    pub fn max_order(&self) -> MaxOrder {
        self.max_order
    }

    /// The index of the target column for the causal analysis.
    pub fn target_col(&self) -> usize {
        self.target_col
    }
}

impl fmt::Display for SurdConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SurdConfig(max_order: {}, target_col: {})",
            self.max_order, self.target_col
        )
    }
}
