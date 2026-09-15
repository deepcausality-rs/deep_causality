/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Mapping a value through a function, and summarising a sample as one.

use crate::UncertainScalar;
use crate::{Node, SampleSession, Uncertain, UncertainBool, UncertainError};
use deep_causality_stats::{mean, std_dev};

impl<R: UncertainScalar> Uncertain<R> {
    /// Summarises a sample as a normal `Uncertain`, by its mean and unbiased `n − 1` variance.
    ///
    /// The statistics come from `deep_causality_stats`; only the degenerate answers are this
    /// crate's. An empty sample is a point at zero and a single sample has zero spread, where the
    /// statistics crate refuses both — the right contract there, and the wrong one for a value
    /// whose job is to summarise whatever it is handed.
    pub fn from_samples(samples: &[R]) -> Self {
        if samples.is_empty() {
            return Self::point(R::zero());
        }
        // `unwrap_or` here is unreachable and kept only as a total expression: `mean` refuses
        // exactly one input, the empty slice, and the guard above has already returned for it.
        // Recorded rather than removed because a defect audit of this delegation
        // (`unified-math-next` task 5.19) found no test could distinguish the sentinel — which is
        // the correct outcome for a branch no input reaches, and worth saying so that the next
        // reader does not go looking for the missing test.
        let mean = mean(samples).unwrap_or_else(|_| R::zero());
        // A single sample has no dispersion to estimate; `std_dev` says so with
        // `InsufficientSamples`, and zero is what that means for a summary.
        let std_dev = std_dev(samples).unwrap_or_else(|_| R::zero());
        Self::normal(mean, std_dev)
    }

    /// The fraction of `num_samples` draws that exceed `threshold`.
    pub fn estimate_probability_exceeds(
        &self,
        session: &SampleSession,
        threshold: R,
        num_samples: usize,
    ) -> Result<R, UncertainError> {
        if num_samples == 0 {
            return Ok(R::zero());
        }
        let samples = self.samples_from(session, num_samples)?;
        let count = samples.iter().filter(|&&s| s > threshold).count();
        crate::ratio(count, num_samples)
    }

    /// Maps each draw through `func`, lazily.
    ///
    /// The function is `R -> R`: it sees the caller's scalar and returns it, with no narrowing
    /// through a fixed precision on either side. That was the shape of the old `f64`-typed closure
    /// boundary, and it is gone.
    ///
    /// `func` is a plain function pointer, so the graph stores no trait object and the call is
    /// static. A **capturing** closure therefore does not fit, and should not: a captured parameter
    /// is opaque to the sampler and to the quasi-Monte-Carlo pre-pass, while the same parameter put
    /// into the graph is not. `x.map(|v| v * k)` is `x * Uncertain::point(k)`.
    pub fn map(&self, func: fn(R) -> R) -> Self {
        Self::from_root_node(Node::FunctionOpReal {
            func,
            operand: self.root_node().clone(),
        })
    }

    /// Maps each draw through a predicate, giving the Boolean carrier over the same graph.
    ///
    /// As [`Self::map`], a function pointer rather than a stored closure.
    pub fn map_to_bool(&self, func: fn(R) -> bool) -> UncertainBool<R> {
        UncertainBool::from_root_node(Node::FunctionOpBool {
            func,
            operand: self.root_node().clone(),
        })
    }
}
