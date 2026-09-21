/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Teloid;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};
// activation_predicate is a function pointer hence PartialEq cannot be derived
// and therefore must be implemented manually.

impl<D, S, T, ST> PartialEq for Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<D, S, T, ST> Eq for Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
}
