/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::*;

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
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

    pub fn causal_collection(&self) -> &Option<Arc<Vec<CausaloidId>>> {
        &self.causal_coll
    }

    pub fn causal_graph(&self) -> &Option<Arc<CausaloidGraph<CausaloidId>>> {
        &self.causal_graph
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn causal_fn(&self) -> &Option<CausalFn<I, O>> {
        &self.causal_fn
    }
}
