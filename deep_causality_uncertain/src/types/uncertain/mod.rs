/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    BernoulliParams, ComputationNode, DistributionEnum, NormalDistributionParams, UncertainError,
    UniformDistributionParams, sprt_eval,
};

use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::types::computation::node::NodeId; // Added this import

mod uncertain_arithmetic;
mod uncertain_comparison;
mod uncertain_logic;
mod uncertain_sampling;
mod uncertain_statistics;

// A single static counter for all Uncertain instances to generate unique IDs.
static NEXT_UNCERTAIN_ID: AtomicUsize = AtomicUsize::new(0);

/// A type representing a value with inherent uncertainty, modeled as a probability distribution.
#[derive(Clone, Debug)]
pub struct Uncertain<T> {
    id: usize,
    root_node: Arc<ComputationNode>,
    _phantom: PhantomData<T>,
}

impl<T> Uncertain<T> {
    /// Creates a new `Uncertain` value from a computation graph represented by a root node.
    fn from_root_node(root_node: ComputationNode) -> Self {
        Self {
            id: NEXT_UNCERTAIN_ID.fetch_add(1, Ordering::Relaxed),
            root_node: Arc::new(root_node),
            _phantom: PhantomData,
        }
    }

    pub fn conditional(condition: Uncertain<bool>, if_true: Self, if_false: Self) -> Self {
        Self::from_root_node(ComputationNode::ConditionalOp {
            node_id: NodeId::new(), // Added node_id
            condition: Box::new((*condition.root_node).clone()),
            if_true: Box::new((*if_true.root_node).clone()),
            if_false: Box::new((*if_false.root_node).clone()),
        })
    }
}

impl<T: Copy> Uncertain<T> {
    pub fn id(&self) -> usize {
        self.id
    }
}

// Constructors
impl Uncertain<f64> {
    pub fn point(value: f64) -> Self {
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Point(value),
        })
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        let params = NormalDistributionParams { mean, std_dev };
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Normal(params),
        })
    }

    pub fn uniform(low: f64, high: f64) -> Self {
        let params = UniformDistributionParams { low, high };
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Uniform(params),
        })
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        Self::from_root_node(ComputationNode::FunctionOp {
            node_id: NodeId::new(), // Added node_id
            func: Arc::new(func),
            operand: Box::new((*self.root_node).clone()),
        })
    }

    pub fn map_to_bool<F>(&self, func: F) -> Uncertain<bool>
    where
        F: Fn(f64) -> bool + Send + Sync + 'static,
    {
        Uncertain::from_root_node(ComputationNode::FunctionOpBool {
            node_id: NodeId::new(), // Added node_id
            func: Arc::new(func),
            operand: Box::new((*self.root_node).clone()),
        })
    }
}

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
