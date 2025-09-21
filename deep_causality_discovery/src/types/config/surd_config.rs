/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SurdConfig {
    max_order: MaxOrder,
    target_col: usize,
}

impl SurdConfig {
    pub fn new(max_order: MaxOrder, target_col: usize) -> Self {
        Self {
            max_order,
            target_col,
        }
    }

    pub fn max_order(&self) -> MaxOrder {
        self.max_order
    }

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
