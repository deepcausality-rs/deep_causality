/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{SampledValue, Sampler, SequentialSampler, Uncertain, UncertainError};

// Sampling impl for Uncertain<f64>
impl Uncertain<f64> {
    pub fn sample(&self) -> Result<f64, UncertainError> {
        let sampler = SequentialSampler;
        match sampler.sample(self.graph.as_ref())? {
            SampledValue::Float(f) => Ok(f),
            _ => Err(UncertainError::UnsupportedTypeError(
                "Expected f64, found bool".to_string(),
            )),
        }
    }

    pub fn take_samples(&self, n: usize) -> Result<Vec<f64>, UncertainError> {
        (0..n).map(|_| self.sample()).collect()
    }
}

// Sampling impl for Uncertain<bool>
impl Uncertain<bool> {
    pub fn sample(&self) -> Result<bool, UncertainError> {
        let sampler = SequentialSampler;
        match sampler.sample(self.graph.as_ref())? {
            SampledValue::Bool(b) => Ok(b),
            _ => Err(UncertainError::UnsupportedTypeError(
                "Expected bool, found f64".to_string(),
            )),
        }
    }

    pub fn take_samples(&self, n: usize) -> Result<Vec<bool>, UncertainError> {
        (0..n).map(|_| self.sample()).collect()
    }
}
