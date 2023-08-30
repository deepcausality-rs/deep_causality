// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<'l, D, S, T, ST, V> Identifiable for Causaloid<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V> + Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
