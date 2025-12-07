/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    Assumption, Causaloid, Context, Datable, Model, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Model<I, O, D, S, T, ST, SYM, VS, VT>
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
    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn assumptions(&self) -> &Option<Arc<Vec<Assumption>>> {
        &self.assumptions
    }

    /// Returns a reference to the root `Causaloid` of the model.
    ///
    /// The `Causaloid` encapsulates the core causal logic of the model.
    pub fn causaloid(
        &self,
    ) -> &Arc<Causaloid<I, O, (), Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>> {
        &self.causaloid
    }

    pub fn context(&self) -> &Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>> {
        &self.context
    }
}
