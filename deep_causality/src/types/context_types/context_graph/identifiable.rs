use std::hash::Hash;
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use crate::prelude::{Context, Datable, Identifiable, SpaceTemporal, Spatial, Temporable};

impl<'l, D, S, T, ST, V> Identifiable for Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}
