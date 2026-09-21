/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Teloid;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};

impl<D, S, T, ST> std::fmt::Display for Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
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
