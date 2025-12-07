/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalFn, Causaloid, ContextualCausalFn};
use std::fmt::Debug;

// Constructors
#[allow(clippy::type_complexity)]
impl<I, O, PS, C> Causaloid<I, O, PS, C>
where
    I: Default,
    O: Default + Debug,
    PS: Default + Clone,
    C: Clone,
{
    pub fn set_causal_fn(&mut self, causal_fn: CausalFn<I, O>) {
        self.causal_fn = Some(causal_fn);
    }

    pub fn set_context_causal_fn(&mut self, context_causal_fn: ContextualCausalFn<I, O, PS, C>) {
        self.context_causal_fn = Some(context_causal_fn);
    }

    pub fn set_context(&mut self, context: C) {
        self.context = Some(context);
    }

    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }
}
