/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Struct to hold the parameters for a Uniform distribution.
#[derive(Debug, Clone, Copy)]
pub struct UniformDistributionParams {
    pub low: f64,
    pub high: f64,
}

impl UniformDistributionParams {
    pub fn new(low: f64, high: f64) -> Self {
        Self { low, high }
    }
}
