// SPDX-License-Identifier: MIT
// Copyright (c) "2023" The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{Context, Datable, Identifiable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

impl<D, S, T, ST, SYM, V> Identifiable for Context<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}
