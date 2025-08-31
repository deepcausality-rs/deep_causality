/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::computation::copy_graph_and_get_remapped_root;
use crate::{
    BernoulliParams, ComputationNode, DistributionEnum, NormalDistributionParams, UncertainError,
    UncertainGraph, UniformDistributionParams, sprt_eval,
};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use ultragraph::{GraphMut, GraphView, UltraGraph};

mod uncertain_arithmetic;
mod uncertain_comparison;
mod uncertain_getters;
mod uncertain_logic;
mod uncertain_sampling;
mod uncertain_statistics;

// A single static counter for all Uncertain instances to generate unique IDs.
static NEXT_UNCERTAIN_ID: AtomicUsize = AtomicUsize::new(0);

/// A type representing a value with inherent uncertainty, modeled as a probability distribution.
#[derive(Clone)]
pub struct Uncertain<T> {
    id: usize,
    graph: Arc<UncertainGraph>,
    _phantom: PhantomData<T>,
}

impl<T> Uncertain<T> {
    /// Creates a new `Uncertain` value from a computation graph.
    /// This function copies all nodes and edges from the input graph.
    fn from_graph(source_graph: UncertainGraph) -> Self {
        let mut new_graph = UltraGraph::new();
        let mut node_map: HashMap<usize, usize> = HashMap::new();

        // Copy nodes and remap indices
        for (old_idx, node_data) in source_graph.get_all_nodes().iter().enumerate() {
            let new_idx = new_graph
                .add_node((*node_data).clone())
                .expect("Failed to add node during graph copy");
            node_map.insert(old_idx, new_idx);
        }

        // Copy edges with remapped indices
        for old_src_idx in 0..source_graph.number_nodes() {
            if let Some(edges) = source_graph.get_edges(old_src_idx) {
                for (old_target_idx, weight) in edges {
                    let new_src = node_map[&old_src_idx];
                    let new_target = node_map[&old_target_idx];
                    {
                        let _: () = *weight;
                        new_graph.add_edge(new_src, new_target, ())
                    }
                    .expect("Failed to add edge during graph copy");
                }
            }
        }

        // Set the root node of the new graph if it existed in the source graph
        if let Some(old_root_idx) = source_graph.get_root_index() {
            let new_root_idx = node_map[&old_root_idx];
            new_graph
                .add_root_node(new_graph.get_node(new_root_idx).unwrap().clone())
                .expect("Failed to set root node during graph copy");
        }

        Self {
            id: NEXT_UNCERTAIN_ID.fetch_add(1, Ordering::Relaxed),
            graph: Arc::new(new_graph),
            _phantom: PhantomData,
        }
    }
}

// Constructors
impl Uncertain<f64> {
    pub fn point(value: f64) -> Self {
        let mut g = UltraGraph::new();
        let node = ComputationNode::LeafF64(DistributionEnum::Point(value));
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        let mut g = UltraGraph::new();
        let params = NormalDistributionParams { mean, std_dev };
        let node = ComputationNode::LeafF64(DistributionEnum::Normal(params));
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    pub fn uniform(low: f64, high: f64) -> Self {
        let mut g = UltraGraph::new();
        let params = UniformDistributionParams { low, high };
        let node = ComputationNode::LeafF64(DistributionEnum::Uniform(params));
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");

        let op_node = ComputationNode::FunctionOp {
            func: Arc::new(func),
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");

        new_graph
            .add_edge(self_root, op_idx, ())
            .expect("Failed to add edge");

        Self::from_graph(new_graph)
    }

    pub fn map_to_bool<F>(&self, func: F) -> Uncertain<bool>
    where
        F: Fn(f64) -> bool + Send + Sync + 'static,
    {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");

        let op_node = ComputationNode::FunctionOpBool {
            func: Arc::new(func),
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");

        new_graph
            .add_edge(self_root, op_idx, ())
            .expect("Failed to add edge");

        Uncertain::from_graph(new_graph)
    }
}

impl Uncertain<bool> {
    pub fn point(value: bool) -> Self {
        let mut g = UltraGraph::new();
        let node = ComputationNode::LeafBool(DistributionEnum::Point(value));
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    pub fn bernoulli(p: f64) -> Self {
        let mut g = UltraGraph::new();
        let params = BernoulliParams { p };
        let node = ComputationNode::LeafBool(DistributionEnum::Bernoulli(params));
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
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
