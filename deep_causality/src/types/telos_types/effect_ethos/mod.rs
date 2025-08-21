/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod deontic_inference;
mod derive_verdict;
mod resolve_conflicts;

use ultragraph::{GraphMut, TopologicalGraphAlgorithms};

use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use crate::{DeonticError, TagIndex, Teloid, TeloidGraph, TeloidStore};

/// The `EffectEthos` framework provides a reasoning engine for deontic inference.
/// It encapsulates all the necessary components to evaluate a proposed action
/// against a set of teleological norms (Teloids).
#[derive(Clone, Default)]
pub struct EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    teloid_store: TeloidStore<D, S, T, ST, SYM, VS, VT>,
    tag_index: TagIndex,
    teloid_graph: TeloidGraph,
    // Internal flag to ensure the graph has been verified for acyclicity.
    is_verified: bool,
}

// Constructor and management methods
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Creates a new, empty `EffectEthos`.
    pub fn new() -> Self {
        Self {
            teloid_store: TeloidStore::new(),
            tag_index: TagIndex::new(),
            teloid_graph: TeloidGraph::new(),
            is_verified: false, // Graph is empty, so not yet verified.
        }
    }

    /// A facade method to add a new norm to the ethos.
    /// This is the primary way to build the norm base, ensuring consistency.
    /// It adds the Teloid to the store, updates the tag index, and adds the
    /// TeloidID to the graph, invalidating the verification status.
    pub fn add_norm(&mut self, teloid: Teloid<D, S, T, ST, SYM, VS, VT>) {
        let id = teloid.id();
        let tags = teloid.tags().clone();

        self.teloid_store.insert(teloid);
        for tag in tags {
            self.tag_index.add(tag, id);
        }
        self.teloid_graph
            .graph
            .add_node(id)
            .expect("Failed to add node");
        self.is_verified = false; // A modification invalidates prior verification.
    }

    /// Verifies the integrity of the internal TeloidGraph, primarily checking for cycles.
    /// This is a required step before evaluation can proceed.
    pub fn verify_graph(&mut self) -> Result<(), DeonticError> {
        self.teloid_graph.graph.freeze();
        if self.teloid_graph.graph.has_cycle()? {
            self.is_verified = false;
            Err(DeonticError::GraphIsCyclic)
        } else {
            self.is_verified = true;
            Ok(())
        }
    }

    /// Freezes the internal graph for evaluation.
    pub fn freeze(&mut self) {
        self.teloid_graph.graph.freeze();
    }

    /// Unfreezes the internal graph for modification.
    pub fn unfreeze(&mut self) {
        self.teloid_graph.graph.unfreeze();
        self.is_verified = false; // Modifications require re-verification.
    }
}
