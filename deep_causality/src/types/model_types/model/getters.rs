/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Assumption, Causaloid, Context, Datable, Model, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::sync::Arc;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Model<D, S, T, ST, SYM, VS, VT>
where
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

    pub fn causaloid(&self) -> &Arc<Causaloid<D, S, T, ST, SYM, VS, VT>> {
        &self.causaloid
    }

    pub fn context(&self) -> &Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>> {
        &self.context
    }
}
