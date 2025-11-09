/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Causaloid, Context, Datable, IntoEffectValue, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::sync::{Arc, RwLock};

// Constructors
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
    pub fn set_context(&mut self, context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>) {
        self.context = context;
    }
}
