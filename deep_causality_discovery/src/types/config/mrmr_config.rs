/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone)]
pub struct MrmrConfig {
    num_features: usize,
    target_col: usize,
}

impl MrmrConfig {
    pub fn new(num_features: usize, target_col: usize) -> Self {
        Self {
            num_features,
            target_col,
        }
    }

    pub fn num_features(&self) -> usize {
        self.num_features
    }

    pub fn target_col(&self) -> usize {
        self.target_col
    }
}

impl fmt::Display for MrmrConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MrmrConfig(num_features: {}, target_col: {})",
            self.num_features, self.target_col
        )
    }
}
