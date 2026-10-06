/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use alloc::collections::BTreeSet;
use alloc::string::String;
use core::fmt::Display;

/// An element of a causal set, with the elements that precede it.
///
/// A causal set is a set `C` with a relation `≺` that is transitive, irreflexive and locally
/// finite: for any `x, z` only finitely many `y` satisfy `x ≺ y ≺ z`. Transitivity and
/// irreflexivity together exclude cycles (Sorkin 2003, p. 5). There are no coordinates, distances
/// or durations, only the order.
///
/// `predecessors` holds every element `x` with `x ≺ self`: the past of this element, not only its
/// links, the relations not implied by transitivity (Sorkin 2003, p. 5). Irreflexivity is local and
/// [`add_predecessor`](Self::add_predecessor) enforces it. Transitivity, acyclicity across elements
/// and local finiteness are properties of the whole set, which one element cannot see; whoever
/// builds the set keeps them.
///
/// This type is not a `Context` node type: it implements none of the coordinate or time traits.
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let mut e = CausalSetSpacetime::new(3, Some("C".into()));
///
/// assert!(e.add_predecessor(1));
/// assert!(e.add_predecessor(2));
/// assert!(!e.add_predecessor(3)); // an element does not precede itself
///
/// assert!(e.is_after(1));
/// assert_eq!(e.predecessor_count(), 2);
/// ```
///
/// # References
/// - Sorkin, R. D. (2003). Causal Sets: Discrete Gravity (Notes for the Valdivia Summer School).
///   arXiv:gr-qc/0309009, p. 5. Copy:
///   `papers/sorkin_2003_causal_sets_discrete_gravity_arXiv_gr-qc_0309009.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalSetSpacetime {
    /// Unique event identifier
    pub id: ContextoidId,

    /// Optional label or annotation for semantic reasoning
    pub label: Option<String>,

    /// Every element that precedes this one.
    pub predecessors: alloc::collections::BTreeSet<ContextoidId>,
}

impl CausalSetSpacetime {
    pub fn new(id: ContextoidId, label: Option<String>) -> Self {
        Self {
            id,
            label,
            predecessors: BTreeSet::new(),
        }
    }
}

impl CausalSetSpacetime {
    /// Records `parent_id ≺ self`. Returns whether it was added: `false` when `parent_id` is this
    /// element's own id, which irreflexivity forbids, or when it is already recorded.
    pub fn add_predecessor(&mut self, parent_id: ContextoidId) -> bool {
        parent_id != self.id && self.predecessors.insert(parent_id)
    }

    /// Whether `other_id ≺ self`.
    pub fn is_after(&self, other_id: ContextoidId) -> bool {
        self.predecessors.contains(&other_id)
    }

    /// How many elements precede this one: the cardinality of its past.
    pub fn predecessor_count(&self) -> usize {
        self.predecessors.len()
    }
}

impl Display for CausalSetSpacetime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "CausalSetSpacetime {{ id: {}, label: {:?}, predecessors: {:?} }}",
            self.id, self.label, self.predecessors
        )
    }
}
