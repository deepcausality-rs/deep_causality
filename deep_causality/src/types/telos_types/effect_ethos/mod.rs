/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod deontic_inference;
mod derive_verdict;
mod freeze;
mod getters;
mod resolve_conflicts;

mod verify;

use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use crate::{TagIndex, TeloidGraph, TeloidStore};

/// The `EffectEthos` provides a reasoning engine for deontic inference.
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

    /// Build an effect ethos  
    pub fn from(
        teloid_store: TeloidStore<D, S, T, ST, SYM, VS, VT>,
        tag_index: TagIndex,
        teloid_graph: TeloidGraph,
    ) -> Self {
        Self {
            teloid_store,
            tag_index,
            teloid_graph,
            is_verified: false,
        }
    }
}
