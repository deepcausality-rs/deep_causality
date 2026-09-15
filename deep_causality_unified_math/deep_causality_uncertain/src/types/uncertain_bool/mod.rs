/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{BernoulliParams, DistributionEnum, Node, Sample};
use deep_causality_ast::ConstTree;
use deep_causality_rand::RandScalar;

mod uncertain_bool_default;
mod uncertain_bool_hypothesis;
mod uncertain_bool_op_logic;
mod uncertain_bool_part_eq;
mod uncertain_bool_sampling;
mod uncertain_bool_verdict;

/// A truth value with inherent uncertainty, over the same computation graph as [`Uncertain<R>`].
///
/// # Why this is not `Uncertain<bool>`
///
/// It was, while the graph carried its precision in a closed enum and the carrier's type parameter
/// was a label. Once the graph became `ConstTree<Node<R>>`, a Boolean node stopped having a scalar
/// to name: a Bernoulli leaf, a comparison and a logical combination all read a truth value off a
/// tree whose leaves are real, so `bool` is not a value the tree could be parameterised by. Two
/// carriers over one graph is what that fact looks like in the type system.
///
/// # Why it still carries `R`
///
/// Because the tree beneath a Boolean root holds reals: a comparison's threshold is `R`, an
/// arithmetic operand is `R`, and the whole graph is drawn at one scalar. `UncertainBool<R>` is a
/// view of an `R`-carrying graph whose root happens to yield [`Sample::Bool`], not a separate
/// scalar-free structure.
///
/// [`Uncertain<R>`]: crate::Uncertain
#[derive(Clone, Debug)]
pub struct UncertainBool<R> {
    root_node: ConstTree<Node<R>>,
}

impl<R: RandScalar> UncertainBool<R> {
    /// Creates a new `UncertainBool` from a computation graph root.
    pub(crate) fn from_root_node(root_node: Node<R>) -> Self {
        Self {
            root_node: ConstTree::new(root_node),
        }
    }

    /// The root of this value's computation graph. Crate-internal.
    pub(crate) fn root_node(&self) -> &ConstTree<Node<R>> {
        &self.root_node
    }

    /// The graph, by value. Crate-internal; what an operator consumes to build the next node.
    pub(crate) fn into_tree(self) -> ConstTree<Node<R>> {
        self.root_node
    }

    /// A certain truth value.
    pub fn point(value: bool) -> Self {
        Self::from_root_node(Node::Value(Sample::Bool(value)))
    }

    /// A Bernoulli trial with success probability `p`.
    ///
    /// # The probability's resolution
    ///
    /// `p` is stated in the caller's scalar and held as 64-bit fixed point by the underlying
    /// `Bernoulli`, which is what makes `p = 0` and `p = 1` exact. A scalar wider than 64 bits of
    /// significand therefore states a finer probability than the draw honours: the parameter is
    /// respected to a multiple of `2^-64` whatever scalar states it. That is a bound of the
    /// representation, not of the scalar, and widening the scalar does not move it.
    pub fn bernoulli(p: R) -> Self {
        Self::from_root_node(Node::Distribution(DistributionEnum::Bernoulli(
            BernoulliParams::at(p),
        )))
    }

    /// Selects between two truth values by an uncertain condition, per draw.
    pub fn conditional(condition: Self, if_true: Self, if_false: Self) -> Self {
        Self::from_root_node(Node::ConditionalOp {
            condition: condition.root_node,
            if_true: if_true.root_node,
            if_false: if_false.root_node,
        })
    }

    /// The value at the root when it is a certain one, and `false` otherwise.
    ///
    /// A display-boundary convenience, mirroring [`Uncertain::value`](crate::Uncertain::value): a
    /// graph that draws has no single value to report, and returning a draw from a method that
    /// names no session would be a draw no seed reproduces.
    pub fn value(&self) -> bool {
        match self.root_node.value() {
            Node::Value(Sample::Bool(v)) => *v,
            _ => false,
        }
    }
}
