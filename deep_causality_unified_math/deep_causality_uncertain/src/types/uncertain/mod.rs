/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    DistributionEnum, Node, NormalDistributionParams, Sample, UncertainBool,
    UniformDistributionParams,
};

use crate::UncertainScalar;
use deep_causality_ast::ConstTree;

mod uncertain_default;
mod uncertain_map;
mod uncertain_op_arithmetic;
mod uncertain_op_comparison;
mod uncertain_part_eq;
mod uncertain_sampling;
mod uncertain_statistics;
mod uncertain_verdict;

/// A real-valued quantity with inherent uncertainty, modelled as a lazy computation graph.
///
/// # The scalar is a parameter
///
/// `R` is the precision every value in the graph is carried at: a point, a distribution parameter,
/// a comparison threshold, an arithmetic result. It is bounded by `UncertainScalar` — `RealField +
/// FromPrimitive`, blanket-implemented — so a scalar joins by satisfying the algebra and by
/// nothing else. Nothing in this crate names a concrete one.
///
/// # The Boolean sibling
///
/// A comparison or a Bernoulli leaf produces a truth value from a tree of reals, and that is
/// [`UncertainBool<R>`], not `Uncertain<bool>`. The two are separate types over the same
/// `ConstTree<Node<R>>` — see the crate documentation for why one type cannot be both.
#[derive(Clone, Debug)]
pub struct Uncertain<R> {
    root_node: ConstTree<Node<R>>,
}

impl<R: UncertainScalar> Uncertain<R> {
    /// Creates a new `Uncertain` value from a computation graph represented by a root node.
    pub(crate) fn from_root_node(root_node: Node<R>) -> Self {
        Self {
            root_node: ConstTree::new(root_node),
        }
    }

    /// The root of this value's computation graph.
    ///
    /// Crate-internal: a [`QmcSampler`](crate::QmcSampler) is built from an `&Uncertain<R>` via
    /// [`QmcSampler::new`](crate::QmcSampler::new), so callers never need the raw root.
    pub(crate) fn root_node(&self) -> &ConstTree<Node<R>> {
        &self.root_node
    }

    /// The graph, by value. Crate-internal; what an operator consumes to build the next node.
    pub(crate) fn into_tree(self) -> ConstTree<Node<R>> {
        self.root_node
    }

    /// A certain value, carried losslessly at `R`'s precision.
    pub fn point(value: R) -> Self {
        Self::from_root_node(Node::Value(Sample::Real(value)))
    }

    /// A normal (Gaussian) distribution at `R`'s precision.
    pub fn normal(mean: R, std_dev: R) -> Self {
        Self::from_root_node(Node::Distribution(DistributionEnum::Normal(
            NormalDistributionParams { mean, std_dev },
        )))
    }

    /// A uniform distribution on `[low, high)` at `R`'s precision.
    pub fn uniform(low: R, high: R) -> Self {
        Self::from_root_node(Node::Distribution(DistributionEnum::Uniform(
            UniformDistributionParams { low, high },
        )))
    }

    /// Selects between two values by an uncertain condition, per draw.
    ///
    /// The condition is an [`UncertainBool<R>`] over the same scalar, so all three operands are one
    /// tree and one sample decides the branch and evaluates it.
    pub fn conditional(condition: UncertainBool<R>, if_true: Self, if_false: Self) -> Self {
        Self::from_root_node(Node::ConditionalOp {
            condition: condition.into_tree(),
            if_true: if_true.root_node,
            if_false: if_false.root_node,
        })
    }

    /// The value at the root when it is a certain one, and `R::zero()` otherwise.
    ///
    /// A display-boundary convenience for a graph that was built from [`Self::point`]. A graph that
    /// draws has no single value to report and gives zero rather than a draw, because returning a
    /// draw from a method that does not name a session would be a draw no seed reproduces.
    pub fn value(&self) -> R {
        match self.root_node.value() {
            Node::Value(Sample::Real(v)) => *v,
            _ => R::zero(),
        }
    }
}
