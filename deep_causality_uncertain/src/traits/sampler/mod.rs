/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{ComputationNode, SampledValue, UncertainError};
use std::sync::Arc;

/// A trait for sampling strategies.
pub trait Sampler {
    /// Generates a single sample from the computation graph.
    fn sample(&self, root_node: &Arc<ComputationNode>) -> Result<SampledValue, UncertainError>;
}
