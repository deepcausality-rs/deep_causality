/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ComputationNode, LogicalOperator, SampledValue, Sampler, UncertainError, UncertainGraph,
};
use rand::rng;
use std::collections::HashMap;
use ultragraph::{GraphTraversal, GraphView, TopologicalGraphAlgorithms};

/// A basic, single-threaded sampler.
#[derive(Default)]
pub struct SequentialSampler;

impl Sampler for SequentialSampler {
    fn sample(&self, graph: &UncertainGraph) -> Result<SampledValue, UncertainError> {
        let mut g = graph.clone();
        if !g.is_frozen() {
            g.freeze();
        }

        let sorted_nodes = g
            .topological_sort()?
            .expect("Computation graph has a cycle");
        let mut context: HashMap<usize, SampledValue> = HashMap::new();
        let mut rng = rng();

        for node_idx in sorted_nodes {
            let node = g.get_node(node_idx).unwrap();
            let value = match node {
                ComputationNode::LeafF64(dist) => SampledValue::Float(dist.sample(&mut rng)?),
                ComputationNode::LeafBool(dist) => SampledValue::Bool(dist.sample(&mut rng)?),

                ComputationNode::ArithmeticOp { op } => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: Arithmetic op requires float inputs"),
                        })
                        .collect();
                    SampledValue::Float(op.apply(inputs[0], inputs[1]))
                }

                ComputationNode::ComparisonOp { op, threshold } => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: Comparison op requires float inputs"),
                        })
                        .collect();
                    SampledValue::Bool(op.apply(inputs[0], *threshold))
                }

                ComputationNode::LogicalOp { op } => {
                    let inputs: Vec<bool> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Bool(b) => b,
                            _ => panic!("Type error: Logical op requires boolean inputs"),
                        })
                        .collect();

                    let result = match op {
                        LogicalOperator::And => inputs[0] && inputs[1],
                        LogicalOperator::Or => inputs[0] || inputs[1],
                        LogicalOperator::Not => !inputs[0],
                    };
                    SampledValue::Bool(result)
                }

                ComputationNode::FunctionOp { func } => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: Function op requires float input"),
                        })
                        .collect();
                    SampledValue::Float(func(inputs[0]))
                }

                ComputationNode::NegationOp => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: Negation op requires float input"),
                        })
                        .collect();
                    SampledValue::Float(-inputs[0])
                }

                ComputationNode::FunctionOpBool { func } => {
                    let inputs: Vec<f64> = g
                        .inbound_edges(node_idx)?
                        .map(|p_idx| match context[&p_idx] {
                            SampledValue::Float(f) => f,
                            _ => panic!("Type error: FunctionOpBool requires float input"),
                        })
                        .collect();
                    SampledValue::Bool(func(inputs[0]))
                }
            };
            context.insert(node_idx, value);
        }

        let root_idx = g.get_root_index().expect("Graph has no root node");
        Ok(context[&root_idx])
    }
}
