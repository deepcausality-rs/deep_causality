/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::Identifiable;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};

use crate::Teloid;

impl<D, S, T, ST, VS, VT> Identifiable for Teloid<D, S, T, ST, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    VS: Clone,
    VT: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
