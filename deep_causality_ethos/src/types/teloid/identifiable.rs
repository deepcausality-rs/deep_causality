/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Datable, Identifiable, SpaceTemporal, Spatial, Symbolic, Temporal};

use crate::Teloid;

impl<D, S, T, ST, SYM, VS, VT> Identifiable for Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
