// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use super::*;

// Getters
impl<'l, D, S, T, ST> Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    pub fn active(&self) -> bool {
        self.active.get()
    }
    pub fn causal_collection(&self) -> Option<Vec<Causaloid<'l, D, S, T, ST>>> {
        self.causal_coll.clone()
    }
    pub fn causal_graph(&self) -> Option<CausaloidGraph<Causaloid<'l, D, S, T, ST>>> {
        self.causal_graph.clone()
    }
    pub fn last_obs(&self) -> NumericalValue {
        self.last_obs.get()
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn context(&self) -> Option<&'l Context<'l, D, S, T, ST>> {
        self.context
    }
}

