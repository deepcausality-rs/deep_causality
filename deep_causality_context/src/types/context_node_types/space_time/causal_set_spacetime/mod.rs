/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
mod identifiable;

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
/// [`predecessors`](Self::predecessors) holds every element `x` with `x ≺ self`: the past of this
/// element, not only its links, the relations not implied by transitivity (Sorkin 2003, p. 5).
/// Irreflexivity is local: [`add_predecessor`](Self::add_predecessor) is the only way to extend the
/// past, and it refuses the element's own id. No method changes the id. Transitivity, acyclicity
/// across elements and local finiteness are properties of the whole set, which one element cannot
/// see; whoever builds the set keeps them.
///
/// It implements none of the coordinate or time traits, so it cannot fill a `Context`'s spacetime
/// slot. A context carries it in its data slot instead, as `Data<CausalSetSpacetime>`. The
/// [`Default`] element has id 0, no label and an empty past, which keeps irreflexivity.
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
/// The past is not writable from outside the type:
///
/// ```compile_fail
/// use deep_causality_context::*;
///
/// let mut e = CausalSetSpacetime::new(3, None);
/// e.predecessors.insert(3); // private field
/// ```
///
/// Nor can it change the id to one it records as a predecessor:
///
/// ```compile_fail
/// use deep_causality_context::*;
///
/// let mut e = CausalSetSpacetime::new(3, None);
/// e.add_predecessor(5);
/// e.id = 5; // private field
/// ```
///
/// # References
/// - Sorkin, R. D. (2003). Causal Sets: Discrete Gravity (Notes for the Valdivia Summer School).
///   arXiv:gr-qc/0309009, p. 5. Copy:
///   `papers/sorkin_2003_causal_sets_discrete_gravity_arXiv_gr-qc_0309009.pdf`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CausalSetSpacetime {
    /// Unique event identifier
    id: ContextoidId,

    /// Optional label or annotation for semantic reasoning
    label: Option<String>,

    /// Every element that precedes this one.
    predecessors: alloc::collections::BTreeSet<ContextoidId>,
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
    /// The label, if one was given.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Every element that precedes this one, in ascending id order.
    pub fn predecessors(&self) -> &BTreeSet<ContextoidId> {
        &self.predecessors
    }

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
