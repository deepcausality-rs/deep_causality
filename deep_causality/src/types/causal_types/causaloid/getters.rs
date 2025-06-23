// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

// Getters
impl<'l,D, S, T, ST, SYM, V>  Causaloid<'l, D, S, T, ST, SYM, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporal<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    SYM: Symbolic + Clone,
    V: Clone,
{
    pub fn active(&self) -> bool {
        self.is_active()
    }
    pub fn causal_collection(&self) -> Option<&CausalVec<'l, D, S, T, ST, SYM, V>> {
        self.causal_coll
    }
    pub fn causal_graph(&self) -> Option<&CausalGraph<'l, D, S, T, ST, SYM, V>> {
        self.causal_graph
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn context(&self) -> Option<&'l Context<D, S, T, ST, SYM, V>> {
        self.context
    }
}
