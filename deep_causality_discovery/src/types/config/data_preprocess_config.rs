/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone)]
pub enum BinningStrategy {
    EqualWidth,
    EqualFrequency,
}

#[derive(Debug, Clone)]
pub enum ColumnSelector {
    All,
    ByIndex(Vec<usize>),
    ByName(Vec<String>), // Note: Requires header resolution
}

/// Configuration for a data discretization pre-processing step.
#[derive(Debug, Clone)]
pub struct PreprocessConfig {
    strategy: BinningStrategy,
    num_bins: usize,
    columns: ColumnSelector,
}

impl PreprocessConfig {
    pub fn new(strategy: BinningStrategy, num_bins: usize, columns: ColumnSelector) -> Self {
        Self {
            strategy,
            num_bins,
            columns,
        }
    }

    pub fn strategy(&self) -> &BinningStrategy {
        &self.strategy
    }

    pub fn num_bins(&self) -> usize {
        self.num_bins
    }

    pub fn columns(&self) -> &ColumnSelector {
        &self.columns
    }
}

impl fmt::Display for PreprocessConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PreprocessConfig(strategy: {:?}, num_bins: {}, columns: {:?})",
            self.strategy, self.num_bins, self.columns
        )
    }
}
