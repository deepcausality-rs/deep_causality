/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Assumption, Causaloid, Context, Datable, Identifiable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use std::sync::Arc;

#[allow(clippy::type_complexity)]
pub struct Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: u64,
    author: String,
    description: String,
    assumptions: Option<Arc<Vec<Assumption>>>,
    causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
}

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
    pub fn new(
        id: u64,
        author: &str,
        description: &str,
        assumptions: Option<Arc<Vec<Assumption>>>,
        causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
        context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
    ) -> Self {
        Self {
            id,
            author: author.to_string(),
            description: description.to_string(),
            assumptions,
            causaloid,
            context,
        }
    }
}

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

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Identifiable for Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
