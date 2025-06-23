// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

// Getters
impl<'l, D, S, T, ST, SYM, VS, VT> Causaloid<'l, D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn active(&self) -> bool {
        self.is_active()
    }
    pub fn causal_collection(&self) -> Option<&CausalVec<'l, D, S, T, ST, SYM, VS, VT>> {
        self.causal_coll
    }
    pub fn causal_graph(&self) -> Option<&CausalGraph<'l, D, S, T, ST, SYM, VS, VT>> {
        self.causal_graph
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn context(&self) -> Option<&'l Context<D, S, T, ST, SYM, VS, VT>> {
        self.context
    }
}
