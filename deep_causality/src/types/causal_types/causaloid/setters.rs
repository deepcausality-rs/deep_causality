/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{
    Causaloid, Context, ContextualCausalDataFn, Datable, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::sync::Arc;

// Constructors
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
    pub fn set_has_context(&mut self, has_context: bool) {
        self.has_context = has_context;
    }

    pub fn set_context(&mut self, context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>) {
        self.context = context;
    }

    pub fn set_context_causal_fn(
        &mut self,
        context_causal_fn: Option<ContextualCausalDataFn<D, S, T, ST, SYM, VS, VT>>,
    ) {
        self.context_causal_fn = context_causal_fn;
    }
}
