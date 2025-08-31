/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::sampler::SampledValue;
use crate::{UncertainError, UncertainGraph};

/// A trait for sampling strategies.
pub trait Sampler {
    /// Generates a single sample from the computation graph.
    fn sample(&self, graph: &UncertainGraph) -> Result<SampledValue, UncertainError>;
}
