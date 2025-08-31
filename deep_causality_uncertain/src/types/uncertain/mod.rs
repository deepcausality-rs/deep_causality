/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ComputationNode, Distribution, NormalDistributionParams, UncertainGraph,
    UniformDistributionParams,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use ultragraph::{GraphMut, UltraGraph};

mod uncertain_arithmetic;
mod uncertain_getters;
mod uncrtain_sampling;

// A single static counter for all Uncertain instances to generate unique IDs.
static NEXT_UNCERTAIN_ID: AtomicUsize = AtomicUsize::new(0);

/// A type representing a value with inherent uncertainty, modeled as a probability distribution.
#[derive(Clone)]
pub struct Uncertain {
    /// A unique identifier for this uncertain value.
    id: usize,
    /// A shared pointer to the underlying computation graph.
    graph: Arc<UncertainGraph>,
}

impl Uncertain {
    /// Creates a new `Uncertain` value from a computation graph.
    fn from_graph(graph: UncertainGraph) -> Self {
        Self {
            id: NEXT_UNCERTAIN_ID.fetch_add(1, Ordering::Relaxed),
            graph: Arc::new(graph),
        }
    }

    /// Creates an `Uncertain` value that is a single, certain point.
    pub fn point(value: f64) -> Self {
        let mut g = UltraGraph::new();
        let node = ComputationNode::Leaf {
            dist: Distribution::Point(value),
        };
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    /// Creates an `Uncertain` value from a Normal (Gaussian) distribution.
    pub fn normal(mean: f64, std_dev: f64) -> Self {
        let mut g = UltraGraph::new();
        let params = NormalDistributionParams { mean, std_dev };
        let node = ComputationNode::Leaf {
            dist: Distribution::Normal(params),
        };
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }

    /// Creates an `Uncertain` value from a Uniform distribution.
    pub fn uniform(low: f64, high: f64) -> Self {
        let mut g = UltraGraph::new();
        let params = UniformDistributionParams { low, high };
        let node = ComputationNode::Leaf {
            dist: Distribution::Uniform(params),
        };
        g.add_root_node(node).unwrap();
        Self::from_graph(g)
    }
}
