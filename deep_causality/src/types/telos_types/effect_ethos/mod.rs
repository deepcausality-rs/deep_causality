/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod api;
mod deontic_inference;
mod derive_verdict;
mod freeze;
mod resolve_conflicts;
mod verify;

use crate::{Datable, SpaceTemporal, Spatial, Symbolic, TeloidID, Temporal};
use crate::{TagIndex, TeloidGraph, TeloidStore};
use std::collections::HashMap;
use ultragraph::GraphView;

/// The `EffectEthos` provides a reasoning engine for deontic inference.
/// It encapsulates all the necessary components to evaluate a proposed action
/// against a set of teleological norms (Teloids).
#[derive(Clone, Debug, Default)]
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
    id_to_index_map: HashMap<TeloidID, usize>,
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
            id_to_index_map: HashMap::new(),
            is_verified: false, // Graph is empty, so not yet verified.
        }
    }

    /// Build an effect ethos
    pub fn from(
        teloid_store: TeloidStore<D, S, T, ST, SYM, VS, VT>,
        tag_index: TagIndex,
        teloid_graph: TeloidGraph,
    ) -> Self {
        // Note: This constructor assumes the graph and store are consistent,
        // but it will still require verification. A map is built here for consistency.
        let mut id_to_index_map = HashMap::new();
        for (i, &id) in teloid_graph.graph.get_all_nodes().iter().enumerate() {
            id_to_index_map.insert(*id, i);
        }

        Self {
            teloid_store,
            tag_index,
            teloid_graph,
            id_to_index_map,
            is_verified: false,
        }
    }
}
