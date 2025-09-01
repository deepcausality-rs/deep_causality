/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComputationNode, LogicalOperator, SampledValue, Sampler, UncertainError};
use std::collections::HashMap;
use std::sync::Arc;

/// A basic, single-threaded sampler.
#[derive(Default)]
pub struct SequentialSampler;

// Implementation of the Sampler trait.
impl Sampler for SequentialSampler {
    /// Samples a value from the given root computation node.
    ///
    /// This method initiates the sampling process by evaluating the computation graph
    /// starting from the `root_node`. It uses a `HashMap` for memoization to avoid
    /// recomputing values for the same node multiple times within a single sample operation.
    ///
    /// # Arguments
    ///
    /// * `root_node` - An `Arc` to the root `ComputationNode` of the graph to be sampled.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(SampledValue)` containing the sampled value if the sampling is successful.
    /// - `Err(UncertainError)` if an error occurs during sampling (e.g., type mismatch, distribution error).
    fn sample(&self, root_node: &Arc<ComputationNode>) -> Result<SampledValue, UncertainError> {
        let mut context: HashMap<*const ComputationNode, SampledValue> = HashMap::new();
        // Call the internal method.
        self.evaluate_node(root_node, &mut context, &mut rand::rng())
    }
}

#[allow(clippy::only_used_in_recursion)]
impl SequentialSampler {
    // This function is a private helper method for SequentialSampler.
    /// Recursively evaluates a computation node and its dependencies to produce a `SampledValue`.
    ///
    /// This is a private helper method used by the `sample` method. It performs a depth-first
    /// traversal of the computation graph, evaluating each node based on its type and
    /// its children's evaluated values. It uses a `context` `HashMap` for memoization
    /// to store and retrieve already computed node values, preventing redundant calculations.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the current `ComputationNode` to be evaluated.
    /// * `context` - A mutable reference to a `HashMap` used for memoization. Keys are raw pointers
    ///   to `ComputationNode`s, and values are their `SampledValue`s.
    /// * `rng` - A mutable reference to a random number generator implementing the `rand::Rng` trait,
    ///   used for sampling from distributions.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(SampledValue)` containing the evaluated value of the node.
    /// - `Err(UncertainError)` if an error occurs during evaluation (e.g., type mismatch,
    ///   distribution sampling error).
    fn evaluate_node(
        &self,
        node: &ComputationNode,
        context: &mut HashMap<*const ComputationNode, SampledValue>,
        rng: &mut impl rand::Rng,
    ) -> Result<SampledValue, UncertainError> {
        // Use raw pointer as key for memoization
        let ptr: *const ComputationNode = node as *const ComputationNode;

        if let Some(value) = context.get(&ptr) {
            return Ok(*value);
        }

        let result = match node {
            ComputationNode::LeafF64(dist) => SampledValue::Float(dist.sample(rng)?),
            ComputationNode::LeafBool(dist) => SampledValue::Bool(dist.sample(rng)?),

            ComputationNode::ArithmeticOp { op, lhs, rhs } => {
                let lhs_val = self.evaluate_node(lhs, context, rng)?;
                let rhs_val = self.evaluate_node(rhs, context, rng)?;
                match (lhs_val, rhs_val) {
                    (SampledValue::Float(l), SampledValue::Float(r)) => {
                        SampledValue::Float(op.apply(l, r))
                    }
                    _ => panic!("Type error: Arithmetic op requires float inputs"),
                }
            }

            ComputationNode::ComparisonOp {
                op,
                threshold,
                operand,
            } => {
                let operand_val = self.evaluate_node(operand, context, rng)?;
                match operand_val {
                    SampledValue::Float(o) => SampledValue::Bool(op.apply(o, *threshold)),
                    _ => panic!("Type error: Comparison op requires float input"),
                }
            }

            ComputationNode::LogicalOp { op, operands } => {
                let mut vals = Vec::with_capacity(operands.len());
                for operand_node in operands {
                    match self.evaluate_node(operand_node, context, rng)? {
                        SampledValue::Bool(b) => vals.push(b),
                        _ => return Err(UncertainError::TypeError("Logical op requires boolean inputs".into())),
                    }
                }
                let result = match op {
                    LogicalOperator::Not => {
                        if vals.len() != 1 {
                            return Err(UncertainError::TypeError("NOT expects exactly 1 operand".into()));
                        }
                        !vals[0]
                    }
                    LogicalOperator::And | LogicalOperator::Or | LogicalOperator::NOR | LogicalOperator::XOR => {
                        if vals.len() != 2 {
                            return Err(UncertainError::TypeError("Binary logical op expects exactly 2 operands".into()));
                        }
                        match op {
                            LogicalOperator::And => vals[0] && vals[1],
                            LogicalOperator::Or => vals[0] || vals[1],
                            LogicalOperator::NOR => !(vals[0] || vals[1]),
                            LogicalOperator::XOR => vals[0] ^ vals[1],
                            LogicalOperator::Not => unreachable!(),
                        }
                    }
                };
                SampledValue::Bool(result)
            }

            ComputationNode::FunctionOp { func, operand } => {
                let operand_val = self.evaluate_node(operand, context, rng)?;
                match operand_val {
                    SampledValue::Float(o) => SampledValue::Float(func(o)),
                    _ => panic!("Type error: Function op requires float input"),
                }
            }

            ComputationNode::NegationOp { operand } => {
                let operand_val = self.evaluate_node(operand, context, rng)?;
                match operand_val {
                    SampledValue::Float(o) => SampledValue::Float(-o),
                    _ => panic!("Type error: Negation op requires float input"),
                }
            }

            ComputationNode::FunctionOpBool { func, operand } => {
                let operand_val = self.evaluate_node(operand, context, rng)?;
                match operand_val {
                    SampledValue::Float(o) => SampledValue::Bool(func(o)),
                    _ => panic!("Type error: FunctionOpBool requires float input"),
                }
            }

            ComputationNode::ConditionalOp {
                condition,
                if_true,
                if_false,
            } => {
                let condition_val = match self.evaluate_node(condition, context, rng)? {
                    SampledValue::Bool(b) => b,
                    _ => panic!("Type error: Conditional condition must be boolean"),
                };

                if condition_val {
                    self.evaluate_node(if_true, context, rng)
                } else {
                    self.evaluate_node(if_false, context, rng)
                }?
            }
        };

        context.insert(ptr, result);
        Ok(result)
    }
}
