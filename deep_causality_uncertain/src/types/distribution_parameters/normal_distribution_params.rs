/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Struct to hold the parameters for a Normal (Gaussian) distribution.
#[derive(Debug, Clone, Copy)]
pub struct NormalDistributionParams {
    pub mean: f64,
    pub std_dev: f64,
}

impl NormalDistributionParams {
    pub fn new(mean: f64, std_dev: f64) -> Self {
        Self { mean, std_dev }
    }
}
