/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Struct to hold the parameters for a Uniform distribution, at precision `R`.
///
/// Unbounded for the same reason as [`super::NormalDistributionParams`]: the analytic
/// bound lives on the sampling path, not on the parameter struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UniformDistributionParams<R> {
    pub low: R,
    pub high: R,
}

impl<R> UniformDistributionParams<R> {
    /// Creates a new `UniformDistributionParams` instance.
    ///
    /// # Arguments
    ///
    /// * `low` - The lower bound of the uniform distribution.
    /// * `high` - The upper bound of the uniform distribution.
    ///
    /// # Returns
    ///
    /// A new `UniformDistributionParams` instance.
    pub fn new(low: R, high: R) -> Self {
        Self { low, high }
    }
}

impl<R: core::fmt::Display> core::fmt::Display for UniformDistributionParams<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UniformDistributionParams {{ low: {:.4} , high: {:.4} }}",
            self.low, self.high
        )
    }
}
