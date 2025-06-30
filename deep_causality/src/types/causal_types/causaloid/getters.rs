/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::*;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Causaloid<D, S, T, ST, SYM, VS, VT>
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
    pub fn context(&self) -> &Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>> {
        &self.context
    }

    pub fn causal_collection(&self) -> &Option<Arc<CausalVec<D, S, T, ST, SYM, VS, VT>>> {
        &self.causal_coll
    }

    pub fn causal_graph(&self) -> &Option<Arc<CausalGraph<D, S, T, ST, SYM, VS, VT>>> {
        &self.causal_graph
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn has_context(&self) -> bool {
        self.has_context
    }
}
