/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ComputationNode;
use crate::Sampler;
use crate::{UncertainError, UncertainGraph};
use rand::rng;
use std::collections::HashMap;
use ultragraph::{GraphTraversal, GraphView, TopologicalGraphAlgorithms};

/// A basic, single-threaded sampler.
pub struct SequentialSampler;

impl Sampler for SequentialSampler {
    /// Evaluates the graph by performing a topological sort and then visiting each
    /// node in order, calculating its value based on its dependencies.
    fn sample(&self, graph: &UncertainGraph) -> Result<f64, UncertainError> {
        // A mutable copy is needed to freeze for high-performance traversal.
        let mut g = graph.clone();
        if !g.is_frozen() {
            g.freeze();
        }

        // The topological sort guarantees that we visit nodes only after their
        // dependencies have been evaluated.
        let sorted_nodes = g.topological_sort().unwrap().unwrap();

        // The context for this single sample run. It memoizes results to ensure
        // that nodes used multiple times in a graph are sampled only once.
        let mut context: HashMap<usize, f64> = HashMap::new();
        let mut rng = rng();

        for node_idx in sorted_nodes {
            let node = g.get_node(node_idx).unwrap();
            let value = match node {
                ComputationNode::Leaf { dist } => dist.sample(&mut rng)?,
                ComputationNode::BinaryOp { op } => {
                    // Because of the topological sort, our inputs are guaranteed to be in the context.
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)
                        .unwrap()
                        .map(|parent_idx| context[&parent_idx])
                        .collect();

                    // For this basic implementation, we assume binary operators have exactly two inputs.
                    op.apply(inputs[0], inputs[1])
                }
            };
            context.insert(node_idx, value);
        }

        // The final result is the value of the root node.
        let root_idx = g.get_root_index().expect("Graph has no root node");
        Ok(context[&root_idx])
    }
}
