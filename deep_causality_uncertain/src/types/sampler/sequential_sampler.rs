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

/// A value produced during a sample run. Can be a float or a boolean.
#[derive(Debug, Clone, Copy)]
enum SampledValue {
    Float(f64),
    Bool(bool),
}

/// A basic, single-threaded sampler.
pub struct SequentialSampler;

impl Sampler for SequentialSampler {
    /// Evaluates the graph by performing a topological sort and then visiting each
    /// node in order, calculating its value based on its dependencies.
    fn sample(&self, graph: &UncertainGraph) -> Result<f64, UncertainError> {
        let mut g = graph.clone();
        if !g.is_frozen() {
            g.freeze();
        }

        let sorted_nodes = g.topological_sort().unwrap().unwrap();

        // The context now holds `SampledValue` to support both floats and booleans.
        let mut context: HashMap<usize, SampledValue> = HashMap::new();
        let mut rng = rng();

        for node_idx in sorted_nodes {
            let node = g.get_node(node_idx).unwrap();
            let value = match node {
                ComputationNode::Leaf { dist } => SampledValue::Float(dist.sample(&mut rng)?),

                ComputationNode::ArithmeticOp { op } => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)
                        .unwrap()
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: Arithmetic operation requires float inputs"),
                        })
                        .collect();
                    SampledValue::Float(op.apply(inputs[0], inputs[1]))
                }

                // The implementation for these operators will be completed in the next step
                // as they require the Uncertain type to be generic over <T>.
                ComputationNode::ComparisonOp { .. } => {
                    unimplemented!("Comparison operators not yet implemented in sampler")
                }

                ComputationNode::LogicalOp { .. } => {
                    unimplemented!("Logical operators not yet implemented in sampler")
                }
            };
            context.insert(node_idx, value);
        }

        let root_idx = g.get_root_index().expect("Graph has no root node");
        match context[&root_idx] {
            SampledValue::Float(f) => Ok(f),
            SampledValue::Bool(_) => Err(UncertainError::UnsupportedTypeError(
                "Cannot sample a boolean value as f64. Use a conditional evaluation method instead.".to_string(),
            )),
        }
    }
}
