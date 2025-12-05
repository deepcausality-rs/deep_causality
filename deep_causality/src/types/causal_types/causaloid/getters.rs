/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::*;

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default,
    O: Default + Debug,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn context(&self) -> &Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>> {
        &self.context
    }

    pub fn causal_collection(&self) -> &Option<Arc<Vec<Self>>> {
        &self.causal_coll
    }

    pub fn causal_graph(&self) -> &Option<Arc<CausaloidGraph<Self>>> {
        &self.causal_graph
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
