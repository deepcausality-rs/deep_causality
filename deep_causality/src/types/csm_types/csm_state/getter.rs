/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CausalState, Causaloid, Context, Datable, IntoEffectValue, PropagatingEffect, SpaceTemporal,
    Spatial, Symbolic, Temporal, UncertainParameter,
};
use std::sync::{Arc, RwLock};

impl<I, O, D, S, T, ST, SYM, VS, VT> CausalState<I, O, D, S, T, ST, SYM, VS, VT>
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
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn version(&self) -> usize {
        self.version
    }

    pub fn data(&self) -> &PropagatingEffect {
        &self.data
    }

    pub fn causaloid(&self) -> &Causaloid<I, O, D, S, T, ST, SYM, VS, VT> {
        &self.causaloid
    }

    #[allow(clippy::type_complexity)]
    pub fn context(&self) -> &Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>> {
        self.causaloid.context()
    }

    pub fn uncertain_parameter(&self) -> &Option<UncertainParameter> {
        &self.uncertain_parameter
    }
}
