// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<D, S, T, ST, V> PartialEq for Causaloid<'_, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
