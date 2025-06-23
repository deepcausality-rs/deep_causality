// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<D, S, T, ST, SYM, V> PartialEq for Causaloid<'_, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
