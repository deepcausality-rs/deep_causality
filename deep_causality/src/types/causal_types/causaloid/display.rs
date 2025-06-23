// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<D, S, T, ST, SYM, V> Display for Causaloid<'_, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<D, S, T, ST, SYM, V> Debug for Causaloid<'_, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<D, S, T, ST, SYM, V> Causaloid<'_, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Causaloid id: {} \n Causaloid type: {} \n description: {} is active: {} has context: {}",
               self.id,
               self.causal_type,
               self.description,
               self.is_active(),
               self.has_context,
        )
    }
}
