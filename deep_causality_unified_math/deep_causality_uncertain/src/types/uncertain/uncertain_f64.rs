/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Uncertain, UncertainError, UncertainNodeContent};
use std::sync::Arc;

// `point` / `normal` / `uniform` are the shared generic constructors in `uncertain_real`
// (a single impl, so `Uncertain::normal(0.0, 1.0)` still infers `f64`). The methods below
// are f64-specific.
impl Uncertain<f64> {
    /// Summarises a sample as a normal `Uncertain`, by its mean and unbiased `n − 1` variance.
    ///
    /// The statistics come from `deep_causality_stats`; only the degenerate answers are this
    /// crate's. An empty sample is a point at zero and a single sample has zero spread, where the
    /// statistics crate refuses both — the right contract there, and the wrong one for a value
    /// whose job is to summarise whatever it is handed.
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::point(0.0);
        }
        // `unwrap_or` here is unreachable and kept only as a total expression: `mean` refuses
        // exactly one input, the empty slice, and the guard above has already returned for it.
        // Recorded rather than removed because a defect audit of this delegation
        // (`unified-math-next` task 5.19) found no test could distinguish the sentinel — which is
        // the correct outcome for a branch no input reaches, and worth saying so that the next
        // reader does not go looking for the missing test.
        let mean = deep_causality_stats::mean(samples).unwrap_or(0.0);
        // A single sample has no dispersion to estimate; `std_dev` says so with
        // `InsufficientSamples`, and zero is what that means for a summary.
        let std_dev = deep_causality_stats::std_dev(samples).unwrap_or(0.0);
        Self::normal(mean, std_dev)
    }

    pub fn estimate_probability_exceeds(
        &self,
        threshold: f64,
        num_samples: usize,
    ) -> Result<f64, UncertainError> {
        if num_samples == 0 {
            return Ok(0.0);
        }
        let samples = self.take_samples(num_samples)?;
        let count = samples.iter().filter(|&&s| s > threshold).count();
        Ok(count as f64 / num_samples as f64)
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        Self::from_root_node(UncertainNodeContent::FunctionOpF64 {
            func: Arc::new(func),
            operand: self.root_node.clone(),
        })
    }

    pub fn map_to_bool<F>(&self, func: F) -> Uncertain<bool>
    where
        F: Fn(f64) -> bool + Send + Sync + 'static,
    {
        Uncertain::from_root_node(UncertainNodeContent::FunctionOpBool {
            func: Arc::new(func),
            operand: self.root_node.clone(),
        })
    }
}
