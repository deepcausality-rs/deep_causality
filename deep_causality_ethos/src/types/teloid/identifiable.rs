/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::Identifiable;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};

use crate::Teloid;

impl<D, S, T, ST> Identifiable for Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
