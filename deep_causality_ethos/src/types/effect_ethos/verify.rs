/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DeonticError, EffectEthos};
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};
use ultragraph::TopologicalGraphAlgorithms;
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, VS, VT> EffectEthos<D, S, T, ST, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Verifies the integrity of the internal TeloidGraph, primarily checking for cycles.
    /// This is a required step before evaluation can proceed.
    pub fn verify_graph(&mut self) -> Result<(), DeonticError> {
        self.teloid_graph.graph.freeze();
        if self.teloid_graph.graph.has_cycle()? {
            self.is_verified = false;
            // Fix: Unfreeze graph so the cycle can be fixed
            self.teloid_graph.graph.unfreeze();
            Err(DeonticError::GraphIsCyclic(
                ultragraph::GraphError::GraphContainsCycle,
            ))
        } else {
            self.is_verified = true;
            Ok(())
        }
    }
}
