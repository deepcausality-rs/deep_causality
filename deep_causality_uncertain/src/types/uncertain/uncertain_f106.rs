/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Uncertain<Float106>`: the double-double precision instantiation of the uncertain
//! engine. Construction and sampling mirror `Uncertain<f64>`; the difference is the
//! `DistributionF106` node variant and the `SampledValue::DoubleFloat` extraction, so the
//! sampled value carries its full ~106-bit precision rather than narrowing through f64.

use crate::{
    SampledValue, Sampler, SequentialSampler, Uncertain, UncertainError, with_global_cache,
};
use deep_causality_num::Float106;
use deep_causality_rand::Rng;

// `point` / `normal` / `uniform` are the shared generic constructors in `uncertain_real`.
// The methods below are the `Float106`-specific sampling surface (they extract the
// `SampledValue::DoubleFloat` variant).
impl Uncertain<Float106> {
    /// Draw a sample for a specific sample index; the global cache makes the draw at a
    /// given `(id, index)` reproducible.
    pub fn sample_with_index(&self, sample_index: u64) -> Result<Float106, UncertainError> {
        let key = (self.id, sample_index);

        let computed_value = with_global_cache(|cache| {
            cache.get_or_compute(key, || {
                let sampler = SequentialSampler;
                Sampler::<Float106>::sample(&sampler, &self.root_node)
            })
        })?;

        match computed_value {
            SampledValue::DoubleFloat(d) => Ok(d),
            // A pure-f64 sub-graph widens losslessly into Float106.
            SampledValue::Float(f) => Ok(Float106::from_f64(f)),
            _ => Err(UncertainError::UnsupportedTypeError(
                "Computed value type mismatch: Expected Float106".to_string(),
            )),
        }
    }

    /// Draw a single sample at a random index.
    pub fn sample(&self) -> Result<Float106, UncertainError> {
        let sample_index = deep_causality_rand::rng().random::<u64>();
        self.sample_with_index(sample_index)
    }

    /// Draw `n` independent samples.
    pub fn take_samples(&self, n: usize) -> Result<Vec<Float106>, UncertainError> {
        (0..n)
            .map(|_| {
                let sample_index = deep_causality_rand::rng().random::<u64>();
                self.sample_with_index(sample_index)
            })
            .collect()
    }
}
