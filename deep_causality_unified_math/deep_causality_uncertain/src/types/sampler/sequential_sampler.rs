/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::LeafOrdinals;
use crate::types::sampler::leaf_draws::{AddressedDraws, AmbientDraws, LeafDraws};
use crate::{LogicalOperator, Node, Sample, Sampler, UncertainError};
use deep_causality_ast::ConstTree;
use deep_causality_rand::RandScalar;
use std::collections::HashMap;

/// A basic, single-threaded sampler.
#[derive(Default)]
pub struct SequentialSampler;

// Implementation of the Sampler trait.
impl<R: RandScalar> Sampler<R> for SequentialSampler {
    /// Samples a value from the given root computation node.
    ///
    /// This method initiates the sampling process by evaluating the computation graph
    /// starting from the `root_node`. It uses a `HashMap` for memoization to avoid
    /// recomputing values for the same node multiple times within a single sample operation.
    ///
    /// # Arguments
    ///
    /// * `root_node` - The root of the graph to be sampled.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Sample<R>)` containing the sampled value if the sampling is successful.
    /// - `Err(UncertainError)` if an error occurs during sampling (e.g., type mismatch, distribution error).
    fn sample(
        &self,
        root_node: &ConstTree<Node<R>>,
        _sample_index: u64,
    ) -> Result<Sample<R>, UncertainError> {
        // `_sample_index` is unused on this path: a stateful generator has no notion of an
        // index. `sample_addressed` is the one that takes the index seriously.
        let mut context: HashMap<usize, Sample<R>> = HashMap::new();
        // Host entropy, straight through. There is no seed slot to consult any more: a draw that
        // is meant to be reproducible goes through `sample_addressed`, where the seed is the
        // caller's and arrives in the signature.
        self.evaluate_node(
            root_node,
            &mut context,
            &mut AmbientDraws {
                rng: &mut deep_causality_rand::rng(),
            },
        )
    }
}

impl SequentialSampler {
    /// Evaluates `root_node` at one address: sample `index` of the session seeded `session_seed`,
    /// with `ordinals` assigning a slot to every drawing leaf of that same graph.
    ///
    /// Each drawing leaf gets its own generator, seeded from the three numbers and nothing else, so
    /// the value at a leaf does not depend on how many leaves preceded it in the traversal. Two
    /// graphs sharing a leaf therefore agree about it at a given index.
    ///
    /// The per-call memo still runs, so a leaf reached twice within one sample is drawn once —
    /// `x + x` is twice one draw rather than the sum of two.
    ///
    /// `ordinals` must have been built from this graph. A drawing leaf without one is reported
    /// rather than drawn from elsewhere.
    pub(crate) fn sample_addressed<R: RandScalar>(
        &self,
        root_node: &ConstTree<Node<R>>,
        ordinals: &LeafOrdinals,
        session_seed: u64,
        index: u64,
    ) -> Result<Sample<R>, UncertainError> {
        let mut context: HashMap<usize, Sample<R>> = HashMap::new();
        let mut draws = AddressedDraws {
            seed: session_seed,
            index,
            ordinals,
        };
        self.evaluate_node(root_node, &mut context, &mut draws)
    }
}

#[allow(clippy::only_used_in_recursion)]
impl SequentialSampler {
    /// Recursively evaluates a computation node and its dependencies to produce a [`Sample`].
    ///
    /// Performs a depth-first traversal of the computation graph, evaluating each node from its
    /// children's evaluated values, with `context` memoizing by node identity so a shared
    /// sub-graph is evaluated once per sample.
    ///
    /// # Arguments
    ///
    /// * `node` - The current node to evaluate.
    /// * `context` - The per-sample memo, keyed by `ConstTree::get_id()`.
    /// * `draws` - Where a drawing leaf's entropy comes from.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Sample<R>)` containing the evaluated value of the node.
    /// - `Err(UncertainError)` if an operator is handed a sample of the wrong kind, or a leaf
    ///   cannot be drawn.
    fn evaluate_node<R: RandScalar>(
        &self,
        node: &ConstTree<Node<R>>,
        context: &mut HashMap<usize, Sample<R>>,
        draws: &mut impl LeafDraws<R>,
    ) -> Result<Sample<R>, UncertainError> {
        let current_node_id = node.get_id();

        if let Some(value) = context.get(&current_node_id) {
            return Ok(*value);
        }

        let result = match node.value() {
            Node::Value(v) => *v,
            Node::Distribution(dist) => draws.draw(current_node_id, dist)?,
            Node::ArithmeticOp { op, lhs, rhs } => {
                let lhs_val = self.evaluate_node(lhs, context, draws)?.real()?;
                let rhs_val = self.evaluate_node(rhs, context, draws)?.real()?;
                Sample::Real(op.apply(lhs_val, rhs_val))
            }
            Node::ComparisonOp {
                op,
                threshold,
                operand,
            } => {
                let operand_val = self.evaluate_node(operand, context, draws)?.real()?;
                Sample::Bool(op.apply(operand_val, *threshold))
            }
            Node::LogicalOp { op, operands } => {
                let mut vals = Vec::with_capacity(operands.len());
                for operand_node in operands {
                    vals.push(
                        self.evaluate_node(operand_node, context, draws)?
                            .boolean()?,
                    );
                }
                Sample::Bool(apply_logical(op, &vals)?)
            }
            Node::FunctionOpReal { func, operand } => {
                let operand_val = self.evaluate_node(operand, context, draws)?.real()?;
                Sample::Real(func(operand_val))
            }
            Node::NegationOp { operand } => {
                let operand_val = self.evaluate_node(operand, context, draws)?.real()?;
                Sample::Real(-operand_val)
            }
            Node::FunctionOpBool { func, operand } => {
                let operand_val = self.evaluate_node(operand, context, draws)?.real()?;
                Sample::Bool(func(operand_val))
            }
            Node::ConditionalOp {
                condition,
                if_true,
                if_false,
            } => {
                let condition_val = self.evaluate_node(condition, context, draws)?.boolean()?;

                if condition_val {
                    self.evaluate_node(if_true, context, draws)
                } else {
                    self.evaluate_node(if_false, context, draws)
                }?
            }
        };

        context.insert(current_node_id, result);
        Ok(result)
    }
}

/// Applies a logical operator to its already-evaluated operands.
///
/// Shared by both samplers: the arity check and the truth table are the same whether the operands
/// came from a stateful stream or a Sobol point, and one copy is what keeps them from drifting.
pub(crate) fn apply_logical(op: &LogicalOperator, vals: &[bool]) -> Result<bool, UncertainError> {
    match op {
        LogicalOperator::Not => {
            if vals.len() != 1 {
                return Err(UncertainError::UnsupportedTypeError(
                    "NOT expects exactly 1 operand".into(),
                ));
            }
            Ok(!vals[0])
        }
        LogicalOperator::And
        | LogicalOperator::Or
        | LogicalOperator::NOR
        | LogicalOperator::XOR => {
            if vals.len() != 2 {
                return Err(UncertainError::UnsupportedTypeError(
                    "Binary logical op expects exactly 2 operands".into(),
                ));
            }
            Ok(match op {
                LogicalOperator::And => vals[0] && vals[1],
                LogicalOperator::Or => vals[0] || vals[1],
                LogicalOperator::NOR => !(vals[0] || vals[1]),
                LogicalOperator::XOR => vals[0] ^ vals[1],
                LogicalOperator::Not => unreachable!("handled by the arm above"),
            })
        }
    }
}
