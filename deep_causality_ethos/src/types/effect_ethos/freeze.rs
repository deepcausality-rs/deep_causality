/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EffectEthos;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> EffectEthos<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Checks if the internal graph is frozen for evaluation.
    pub fn is_frozen(&self) -> bool {
        self.teloid_graph.is_frozen()
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
