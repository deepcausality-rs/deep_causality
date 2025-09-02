/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    BernoulliParams, ComputationNode, DistributionEnum, NodeId, Uncertain, UncertainError,
    sprt_eval,
};

impl Uncertain<bool> {
    pub fn point(value: bool) -> Self {
        Self::from_root_node(ComputationNode::LeafBool {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Point(value),
        })
    }

    pub fn bernoulli(p: f64) -> Self {
        let params = BernoulliParams { p };
        Self::from_root_node(ComputationNode::LeafBool {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Bernoulli(params),
        })
    }

    pub fn to_bool(&self, confidence: f64) -> Result<bool, UncertainError> {
        // Default epsilon and max_samples for now. These could be configurable.
        // We pass sample_index 0 as the decision is based on the overall distribution, not a specific sample.
        sprt_eval::evaluate_hypothesis(self, 0.5, confidence, 0.05, 1000, 0)
    }

    /// Evidence-based conditional using hypothesis testing
    pub fn probability_exceeds(
        &self,
        threshold: f64,
        confidence: f64,
        max_samples: usize,
    ) -> Result<bool, UncertainError> {
        sprt_eval::evaluate_hypothesis(self, threshold, confidence, 0.05, max_samples, 0)
    }

    /// Implicit conditional (equivalent to `probability_exceeds(0.5)`) with default confidence and max_samples.
    pub fn implicit_conditional(&self) -> Result<bool, UncertainError> {
        self.probability_exceeds(0.5, 0.95, 1000)
    }

    /// Estimates the probability that this condition is true by taking multiple samples.
    pub fn estimate_probability(&self, num_samples: usize) -> Result<f64, UncertainError> {
        let samples = self.take_samples(num_samples)?;
        if samples.is_empty() {
            Ok(0.0)
        } else {
            Ok(samples.iter().filter(|&&x| x).count() as f64 / samples.len() as f64)
        }
    }
}
