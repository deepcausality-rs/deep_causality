/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quasi-Monte-Carlo sampler.
//!
//! Unlike [`SequentialSampler`](crate::SequentialSampler), which draws each stochastic leaf from
//! a stateful RNG, `QmcSampler` evaluates the graph against a single point of a digitally shifted
//! Sobol sequence: every non-`Point` distribution leaf is assigned a fixed dimension in a
//! deterministic pre-pass, and the sample index selects the Sobol point. Each leaf is then drawn
//! by **inverse-CDF** on its coordinate — the only transform that preserves the sequence's
//! low-discrepancy structure.
//!
//! QMC is sound only for statically-structured trees, so the pre-pass rejects any `ConditionalOp`
//! whose branches draw a different set of distributions — the set of dimensions a sample touches
//! would then depend on the condition's own draw.

use crate::types::sampler::sequential_sampler::apply_logical;
use crate::{DistributionEnum, Node, Sample, Sampler, Uncertain, UncertainBool, UncertainError};
use deep_causality_ast::ConstTree;
use deep_causality_rand::RandScalar;
use deep_causality_rand::{MAX_SOBOL_DIM, SobolSequence};
use deep_causality_stats::{
    bernoulli_inverse_cdf, standard_normal_inverse_cdf_at, uniform_inverse_cdf,
};
use std::collections::{HashMap, HashSet};

/// A Quasi-Monte-Carlo sampler bound to a specific computation graph's dimension layout.
#[derive(Debug, Clone)]
pub struct QmcSampler {
    sobol: SobolSequence,
    /// Stochastic-leaf node id → Sobol dimension index.
    dims: HashMap<usize, usize>,
}

impl QmcSampler {
    /// Builds a `QmcSampler` for `uncertain`'s computation graph, assigning each non-`Point`
    /// distribution leaf a stable dimension. When `seed` is `Some`, the Sobol sequence carries a
    /// seeded digital shift (reproducible randomized QMC); when `None`, it is the raw
    /// (deterministic) sequence.
    ///
    /// Returns `UncertainError::SamplingError` if the tree is not statically structured (a
    /// branch-divergent `ConditionalOp`), or needs more than [`MAX_SOBOL_DIM`] stochastic
    /// dimensions.
    pub fn new<R: RandScalar>(
        uncertain: &Uncertain<R>,
        seed: Option<u64>,
    ) -> Result<Self, UncertainError> {
        Self::from_root_node(uncertain.root_node(), seed)
    }

    /// As [`Self::new`], for the Boolean carrier.
    ///
    /// Two entry points rather than one over a shared root, because the root is not part of either
    /// carrier's surface. A Sobol dimension is a property of a drawing leaf and is assigned
    /// identically here — what differs is only which carrier the caller holds.
    pub fn for_bool<R: RandScalar>(
        uncertain: &UncertainBool<R>,
        seed: Option<u64>,
    ) -> Result<Self, UncertainError> {
        Self::from_root_node(uncertain.root_node(), seed)
    }

    /// Core constructor over a raw computation-graph root. Crate-internal; [`Self::new`] and
    /// [`Self::for_bool`] are the public entry points.
    pub(crate) fn from_root_node<R: RandScalar>(
        root: &ConstTree<Node<R>>,
        seed: Option<u64>,
    ) -> Result<Self, UncertainError> {
        let mut dims = HashMap::new();
        let mut next_dim = 0usize;
        assign_dimensions(root, &mut dims, &mut next_dim)?;

        // The sequence needs at least one dimension even when the tree has no stochastic leaves.
        let dim = next_dim.max(1);
        if dim > MAX_SOBOL_DIM {
            return Err(UncertainError::SamplingError(format!(
                "QMC supports up to {MAX_SOBOL_DIM} stochastic dimensions; this tree needs {next_dim}"
            )));
        }

        let sobol = match seed {
            Some(s) => SobolSequence::new_shifted(dim, s),
            None => SobolSequence::new(dim),
        }
        .map_err(|e| UncertainError::SamplingError(e.to_string()))?;

        Ok(Self { sobol, dims })
    }

    /// The number of stochastic dimensions assigned (the effective QMC dimension `d`).
    pub fn dimension(&self) -> usize {
        self.dims.len()
    }

    /// The `[0,1)` coordinate for the leaf `node_id` at sample `index`.
    ///
    /// Stays `f64`. A Sobol coordinate is an address within the unit cube rather than a value of
    /// the caller's quantity, and the scalar enters at the quantile: widening the address would
    /// buy nothing, while narrowing the *quantile* would cost the tail. Measured at the point it
    /// matters: 2 of 1024 Sobol coordinates saturate the normal quantile when it is evaluated at
    /// a narrow scalar, which is why [`standard_normal_inverse_cdf_at`] evaluates wide and lands
    /// at `R` rather than evaluating at `R`.
    fn coordinate(&self, node_id: usize, index: u64) -> Result<f64, UncertainError> {
        let dim = self.dims.get(&node_id).copied().ok_or_else(|| {
            UncertainError::SamplingError("QMC leaf has no assigned dimension".to_string())
        })?;
        Ok(self.sobol.coordinate(index, dim))
    }

    /// Draws one leaf by inverse-CDF on its Sobol coordinate.
    fn draw_leaf<R: RandScalar>(
        &self,
        node_id: usize,
        index: u64,
        dist: &DistributionEnum<R>,
    ) -> Result<Sample<R>, UncertainError> {
        match dist {
            DistributionEnum::Point(v) => Ok(Sample::Real(*v)),
            DistributionEnum::Normal(params) => {
                let u = self.coordinate(node_id, index)?;
                let z = standard_normal_inverse_cdf_at::<R>(u);
                Ok(Sample::Real(params.mean + params.std_dev * z))
            }
            DistributionEnum::Uniform(params) => {
                let u = self.coordinate(node_id, index)?;
                let u = R::from_f64(u).ok_or_else(|| {
                    UncertainError::SamplingError(
                        "a unit coordinate does not convert to the graph's scalar".to_string(),
                    )
                })?;
                Ok(Sample::Real(uniform_inverse_cdf(
                    u,
                    params.low,
                    params.high,
                )))
            }
            DistributionEnum::Bernoulli(params) => {
                let u = self.coordinate(node_id, index)?;
                Ok(Sample::Bool(bernoulli_inverse_cdf(u, params.p)))
            }
        }
    }

    /// Recursively evaluates a node against the Sobol point at `index`, memoizing by node id so a
    /// shared leaf yields one draw per sample (matching `SequentialSampler`'s semantics).
    fn evaluate_node<R: RandScalar>(
        &self,
        node: &ConstTree<Node<R>>,
        index: u64,
        context: &mut HashMap<usize, Sample<R>>,
    ) -> Result<Sample<R>, UncertainError> {
        let current_node_id = node.get_id();
        if let Some(value) = context.get(&current_node_id) {
            return Ok(*value);
        }

        let result = match node.value() {
            Node::Value(v) => *v,
            Node::Distribution(dist) => self.draw_leaf(current_node_id, index, dist)?,
            Node::ArithmeticOp { op, lhs, rhs } => {
                let lhs_val = self.evaluate_node(lhs, index, context)?.real()?;
                let rhs_val = self.evaluate_node(rhs, index, context)?.real()?;
                Sample::Real(op.apply(lhs_val, rhs_val))
            }
            Node::ComparisonOp {
                op,
                threshold,
                operand,
            } => {
                let operand_val = self.evaluate_node(operand, index, context)?.real()?;
                Sample::Bool(op.apply(operand_val, *threshold))
            }
            Node::LogicalOp { op, operands } => {
                let mut vals = Vec::with_capacity(operands.len());
                for operand_node in operands {
                    vals.push(
                        self.evaluate_node(operand_node, index, context)?
                            .boolean()?,
                    );
                }
                Sample::Bool(apply_logical(op, &vals)?)
            }
            Node::FunctionOpReal { func, operand } => {
                let operand_val = self.evaluate_node(operand, index, context)?.real()?;
                Sample::Real(func(operand_val))
            }
            Node::NegationOp { operand } => {
                let operand_val = self.evaluate_node(operand, index, context)?.real()?;
                Sample::Real(-operand_val)
            }
            Node::FunctionOpBool { func, operand } => {
                let operand_val = self.evaluate_node(operand, index, context)?.real()?;
                Sample::Bool(func(operand_val))
            }
            Node::ConditionalOp {
                condition,
                if_true,
                if_false,
            } => {
                let condition_val = self.evaluate_node(condition, index, context)?.boolean()?;
                if condition_val {
                    self.evaluate_node(if_true, index, context)
                } else {
                    self.evaluate_node(if_false, index, context)
                }?
            }
        };

        context.insert(current_node_id, result);
        Ok(result)
    }
}

impl<R: RandScalar> Sampler<R> for QmcSampler {
    fn sample(
        &self,
        root_node: &ConstTree<Node<R>>,
        sample_index: u64,
    ) -> Result<Sample<R>, UncertainError> {
        let mut context: HashMap<usize, Sample<R>> = HashMap::new();
        self.evaluate_node(root_node, sample_index, &mut context)
    }
}

/// Assigns a Sobol dimension to every non-`Point` distribution leaf, rejecting non-static
/// structure (a branch-divergent `ConditionalOp`).
fn assign_dimensions<R: RandScalar>(
    node: &ConstTree<Node<R>>,
    dims: &mut HashMap<usize, usize>,
    next_dim: &mut usize,
) -> Result<(), UncertainError> {
    match node.value() {
        Node::Value(_) => Ok(()),
        Node::Distribution(dist) => {
            if dist.draws() {
                dims.entry(node.get_id()).or_insert_with(|| {
                    let d = *next_dim;
                    *next_dim += 1;
                    d
                });
            }
            Ok(())
        }
        Node::NegationOp { operand }
        | Node::FunctionOpReal { operand, .. }
        | Node::FunctionOpBool { operand, .. }
        | Node::ComparisonOp { operand, .. } => assign_dimensions(operand, dims, next_dim),
        Node::ArithmeticOp { lhs, rhs, .. } => {
            assign_dimensions(lhs, dims, next_dim)?;
            assign_dimensions(rhs, dims, next_dim)
        }
        Node::LogicalOp { operands, .. } => {
            for operand in operands {
                assign_dimensions(operand, dims, next_dim)?;
            }
            Ok(())
        }
        Node::ConditionalOp {
            condition,
            if_true,
            if_false,
        } => {
            assign_dimensions(condition, dims, next_dim)?;
            assign_dimensions(if_true, dims, next_dim)?;
            assign_dimensions(if_false, dims, next_dim)?;

            let mut true_leaves = HashSet::new();
            collect_stochastic_leaves(if_true, &mut true_leaves);
            let mut false_leaves = HashSet::new();
            collect_stochastic_leaves(if_false, &mut false_leaves);
            if true_leaves != false_leaves {
                return Err(UncertainError::SamplingError(
                    "QMC requires a static stochastic structure: ConditionalOp branches draw \
                     different distributions"
                        .into(),
                ));
            }
            Ok(())
        }
    }
}

/// Collects the node ids of every non-`Point` distribution leaf in `node`'s subtree.
fn collect_stochastic_leaves<R: RandScalar>(node: &ConstTree<Node<R>>, set: &mut HashSet<usize>) {
    match node.value() {
        Node::Distribution(dist) => {
            if dist.draws() {
                set.insert(node.get_id());
            }
        }
        Node::Value(_) => {}
        Node::NegationOp { operand }
        | Node::FunctionOpReal { operand, .. }
        | Node::FunctionOpBool { operand, .. }
        | Node::ComparisonOp { operand, .. } => collect_stochastic_leaves(operand, set),
        Node::ArithmeticOp { lhs, rhs, .. } => {
            collect_stochastic_leaves(lhs, set);
            collect_stochastic_leaves(rhs, set);
        }
        Node::LogicalOp { operands, .. } => {
            for operand in operands {
                collect_stochastic_leaves(operand, set);
            }
        }
        Node::ConditionalOp {
            condition,
            if_true,
            if_false,
        } => {
            collect_stochastic_leaves(condition, set);
            collect_stochastic_leaves(if_true, set);
            collect_stochastic_leaves(if_false, set);
        }
    }
}
