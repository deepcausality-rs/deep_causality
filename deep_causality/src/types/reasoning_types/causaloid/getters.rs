// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

// Getters
impl<'l, D, S, T, ST, V> Causaloid<'l, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V> + Clone,
{
    pub fn active(&self) -> bool {
        self.active.get()
    }
    pub fn causal_collection(&self) -> CausalVec<'l, D, S, T, ST, V> {
        self.causal_coll.clone()
    }
    pub fn causal_graph(&self) -> Option<CausalGraph<'l, D, S, T, ST, V>> {
        self.causal_graph.clone()
    }
    pub fn last_obs(&self) -> NumericalValue {
        self.last_obs.get()
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn context(&self) -> Option<&'l Context<'l, D, S, T, ST, V>> {
        self.context
    }
}
