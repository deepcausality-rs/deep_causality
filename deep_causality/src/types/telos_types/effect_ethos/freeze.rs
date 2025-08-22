/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Datable, EffectEthos, SpaceTemporal, Spatial, Symbolic, Temporal};

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
