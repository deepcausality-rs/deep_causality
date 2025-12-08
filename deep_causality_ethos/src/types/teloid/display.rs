/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Teloid;
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};

impl<D, S, T, ST, SYM, VS, VT> std::fmt::Display for Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Teloid {{ id: {}, action_identifier: \"{}\", modality: {:?}, timestamp: {}, specificity: {}, priority: {}, tags: {:?} }}",
            self.id,
            self.action_identifier,
            self.modality,
            self.timestamp,
            self.specificity,
            self.priority,
            self.tags
        )
    }
}
