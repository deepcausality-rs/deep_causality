/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use alloc::collections::BTreeSet;
use alloc::string::String;
use core::fmt::Display;

/// An event with the set of events it can causally affect.
///
/// The type holds an identifier, a label and a set of identifiers, `causal_links`. It holds no
/// manifold, metric or conformal factor, so it carries no conformal structure; the relation it
/// records is the one [`CausalSetSpacetime`](crate::CausalSetSpacetime) records, seen from the
/// earlier event. Whatever relation the caller enters is what [`can_affect`](Self::can_affect)
/// reports.
///
/// This type is not a `Context` node type: it implements none of the coordinate or time traits.
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let mut n1 = ConformalSpacetime::new(1, Some("Origin".into()));
/// n1.link_to(2);
///
/// assert!(n1.can_affect(2));
/// assert_eq!(n1.fanout(), 1);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ConformalSpacetime {
    /// Unique identifier for this event
    pub id: ContextoidId,

    /// Optional symbolic label (e.g., "i+", "scri", "horizon")
    pub label: Option<String>,

    /// The events this one can affect, as entered.
    pub causal_links: alloc::collections::BTreeSet<ContextoidId>,
}

impl ConformalSpacetime {
    pub fn new(id: ContextoidId, label: Option<String>) -> Self {
        Self {
            id,
            label,
            causal_links: BTreeSet::new(),
        }
    }
}

impl ConformalSpacetime {
    /// Records that this event can affect `other_id`.
    pub fn link_to(&mut self, other_id: ContextoidId) {
        self.causal_links.insert(other_id);
    }

    /// Whether `other_id` is recorded among the events this one can affect.
    pub fn can_affect(&self, other_id: ContextoidId) -> bool {
        self.causal_links.contains(&other_id)
    }

    /// How many events are recorded as affected by this one.
    pub fn fanout(&self) -> usize {
        self.causal_links.len()
    }
}

impl Display for ConformalSpacetime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ConformalSpacetime {{ id: {}, label: {:?} }}",
            self.id, self.label
        )
    }
}
