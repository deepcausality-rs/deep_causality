/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Which slot of the address each drawing leaf occupies.

use crate::{Node, Uncertain, UncertainBool};
use deep_causality_ast::ConstTree;
use deep_causality_rand::RandScalar;
use std::collections::{HashMap, HashSet};

/// The ordinal of every leaf in a tree that draws.
///
/// A draw is addressed by three numbers — the session seed, the sample index, and this ordinal.
/// The first two are the caller's; this supplies the third, and it is the reason a draw can be
/// reproduced without being stored.
///
/// # Why an ordinal and not the node's address
///
/// `ConstTree` identifies a node by `Arc::as_ptr`, which is a heap address. It is a perfectly good
/// key *within one traversal* and a disastrous input to a generator: it differs between two runs of
/// the same program and between two structurally identical trees, so a draw derived from it could
/// not be replayed from a recorded seed. The address is used here to recognise a node already
/// seen, and never leaves this type.
///
/// The ordinal itself comes from the position a leaf occupies in one fixed traversal, so two trees
/// built by the same sequence of constructor calls agree on it however their memory was laid out.
///
/// # Only leaves that draw
///
/// A point distribution returns its value and consumes no entropy, so it takes no ordinal and the
/// ordinals stay dense over the leaves that actually draw. This is the rule
/// [`QmcSampler`](crate::QmcSampler) already applies when it assigns Sobol dimensions; the two
/// pre-passes agree because they ask the same question — `DistributionEnum::draws` — rather than
/// each matching the variants for themselves.
///
/// # No scalar
///
/// An ordinal is a position in a traversal, so the type carries no `R` even though the traversal
/// walks an `R`-carrying tree. One `LeafOrdinals` can therefore be held beside a graph of any
/// scalar, and the constructors are generic rather than the type.
///
/// # No `PartialEq`
///
/// Deliberately absent. The map is keyed by node address, so two structurally identical trees
/// carry equal ordinals under different keys and would compare unequal — the opposite of what a
/// reader comparing them would be asking. What can be compared is what the ordinals produce: the
/// draws.
#[derive(Debug, Clone, Default)]
pub struct LeafOrdinals {
    by_node: HashMap<usize, u64>,
}

impl LeafOrdinals {
    /// Assigns an ordinal to every drawing leaf of `uncertain`'s graph.
    pub fn new<R: RandScalar>(uncertain: &Uncertain<R>) -> Self {
        Self::from_root_node(uncertain.root_node())
    }

    /// As [`Self::new`], for the Boolean carrier.
    pub fn for_bool<R: RandScalar>(uncertain: &UncertainBool<R>) -> Self {
        Self::from_root_node(uncertain.root_node())
    }

    /// Core constructor over a raw graph root.
    ///
    /// Crate-internal, matching [`QmcSampler::from_root_node`](crate::QmcSampler); it lets the
    /// traversal be tested against node shapes no public builder produces.
    pub(crate) fn from_root_node<R: RandScalar>(root: &ConstTree<Node<R>>) -> Self {
        let mut by_node = HashMap::new();
        let mut seen = HashSet::new();
        let mut next = 0u64;
        assign(root, &mut by_node, &mut seen, &mut next);
        Self { by_node }
    }

    /// How many leaves draw.
    pub fn len(&self) -> usize {
        self.by_node.len()
    }

    /// Whether the graph draws at all. True for a graph of point values only.
    pub fn is_empty(&self) -> bool {
        self.by_node.is_empty()
    }

    /// The ordinal of the leaf with this node identity, or `None` if it does not draw.
    pub(crate) fn of_node(&self, node_id: usize) -> Option<u64> {
        self.by_node.get(&node_id).copied()
    }

    /// The ordinals assigned, in ascending order.
    ///
    /// They are `0..len` for any graph, which is what "dense over the drawing leaves" means; a gap
    /// would mean an ordinal was allocated and then not used.
    pub fn assigned(&self) -> Vec<u64> {
        let mut ordinals: Vec<u64> = self.by_node.values().copied().collect();
        ordinals.sort_unstable();
        ordinals
    }
}

/// One deterministic traversal, deduplicating by node identity.
///
/// Every branch is visited in a fixed order, and a node reached twice is skipped the second time.
/// Skipping is what keeps a shared sub-graph linear rather than exponential, and it is safe because
/// the ordinals of a sub-graph are settled by its first visit.
fn assign<R: RandScalar>(
    node: &ConstTree<Node<R>>,
    by_node: &mut HashMap<usize, u64>,
    seen: &mut HashSet<usize>,
    next: &mut u64,
) {
    if !seen.insert(node.get_id()) {
        return;
    }

    match node.value() {
        // Draws nothing.
        Node::Value(_) => {}

        Node::Distribution(distribution) => {
            if distribution.draws() {
                by_node.entry(node.get_id()).or_insert_with(|| {
                    let ordinal = *next;
                    *next += 1;
                    ordinal
                });
            }
        }

        Node::NegationOp { operand }
        | Node::FunctionOpReal { operand, .. }
        | Node::FunctionOpBool { operand, .. }
        | Node::ComparisonOp { operand, .. } => {
            assign(operand, by_node, seen, next);
        }
        Node::ArithmeticOp { lhs, rhs, .. } => {
            assign(lhs, by_node, seen, next);
            assign(rhs, by_node, seen, next);
        }
        Node::LogicalOp { operands, .. } => {
            for operand in operands {
                assign(operand, by_node, seen, next);
            }
        }
        Node::ConditionalOp {
            condition,
            if_true,
            if_false,
        } => {
            assign(condition, by_node, seen, next);
            assign(if_true, by_node, seen, next);
            assign(if_false, by_node, seen, next);
        }
    }
}
