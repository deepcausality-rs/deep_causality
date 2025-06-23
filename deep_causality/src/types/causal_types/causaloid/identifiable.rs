// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl< D, S, T, ST, SYM, V> Identifiable for Causaloid<'_, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
