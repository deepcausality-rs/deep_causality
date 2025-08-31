/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Sampler, SequentialSampler, Uncertain, UncertainError};

// Sampling
impl Uncertain {
    /// Generates a single sample from the distribution using the default sequential sampler.
    /// This operation can fail if the underlying distributions were created with invalid parameters.
    pub fn sample(&self) -> Result<f64, UncertainError> {
        let sampler = SequentialSampler;
        sampler.sample(&self.graph)
    }

    /// Generates `n` samples from the distribution.
    /// If any single sample fails, the entire operation returns an error.
    pub fn take_samples(&self, n: usize) -> Result<Vec<f64>, UncertainError> {
        (0..n).map(|_| self.sample()).collect()
    }
}
