/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Teloid;
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
// activation_predicate is a function pointer hence PartialEq cannot be derived
// and therefore must be implemented manually.

impl<D, S, T, ST, SYM, VS, VT> PartialEq for Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<D, S, T, ST, SYM, VS, VT> Eq for Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
}
