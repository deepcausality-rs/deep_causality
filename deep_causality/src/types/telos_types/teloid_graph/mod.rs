/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod graph;
mod teloidable;

use crate::{TeloidID, TeloidRelation};
use ultragraph::UltraGraphWeighted;

/// A graph structure representing the relationships between Teloids (norms).
///
/// This struct wraps `UltraGraphWeighted`, storing only `TeloidID`s in the nodes
/// for memory efficiency. The edges are weighted with `TeloidRelation` to represent
/// inheritance or defeasance between norms.
#[derive(Clone, Debug, Default)]
pub struct TeloidGraph {
    // The internal graph stores TeloidIDs in nodes and TeloidRelations on edges.
    pub(super) graph: UltraGraphWeighted<TeloidID, TeloidRelation>,
}

impl TeloidGraph {
    /// Creates a new, empty `TeloidGraph`.
    ///
    /// # Returns
    ///
    /// A new `TeloidGraph` instance with an empty internal graph.
    pub fn new() -> Self {
        Self {
            graph: UltraGraphWeighted::new(),
        }
    }

    /// Creates a new `TeloidGraph` with a pre-allocated capacity for its internal graph.
    ///
    /// This can be useful for performance optimization when the approximate number of
    /// nodes (Teloids) is known in advance, reducing the need for reallocations.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity for the internal graph's storage.
    ///
    /// # Returns
    ///
    /// A new `TeloidGraph` instance with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            graph: UltraGraphWeighted::with_capacity(capacity, None),
        }
    }
}
