/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComputationNode, LogicalOperator, SampledValue, Sampler, UncertainError};
use rand::rng;
use std::collections::HashMap;
use std::sync::Arc;

/// A basic, single-threaded sampler.
#[derive(Default)]
pub struct SequentialSampler;

// Implementation of the Sampler trait.
impl Sampler for SequentialSampler {
    // Note: Removed 'pub' qualifier as it's not allowed in trait implementations.
    fn sample(&self, root_node: &Arc<ComputationNode>) -> Result<SampledValue, UncertainError> {
        let mut context: HashMap<*const ComputationNode, SampledValue> = HashMap::new();
        let mut rng = rng();

        // Call the inherent method.
        self.evaluate_node(root_node, &mut context, &mut rng)
    }
}

#[allow(clippy::only_used_in_recursion)]
impl SequentialSampler {
    // Note: Moved evaluate_node here from the trait implementation.
    // This function is now a private helper method for SequentialSampler.
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
                let evaluated_operands: Vec<bool> = operands
                    .iter()
                    .map(|operand_node| {
                        match self.evaluate_node(operand_node, context, rng).unwrap() {
                            SampledValue::Bool(b) => b,
                            _ => panic!("Type error: Logical op requires boolean inputs"),
                        }
                    })
                    .collect();

                let result = match op {
                    LogicalOperator::And => evaluated_operands[0] && evaluated_operands[1],
                    LogicalOperator::Or => evaluated_operands[0] || evaluated_operands[1],
                    LogicalOperator::Not => !evaluated_operands[0],
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
