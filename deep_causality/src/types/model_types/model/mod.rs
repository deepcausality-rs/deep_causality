// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::{Constructor, Getters};
use crate::prelude::{Assumption, Causaloid, Context, Datable, Identifiable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

#[derive(Getters, Constructor)]
pub struct Model<'l, D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    #[getter(name = model_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    author: &'l str,
    description: &'l str,
    assumptions: Option<&'l Vec<&'l Assumption>>,
    causaloid: &'l Causaloid<'l, D, S, T, ST, SYM, VS, VT>,
    context: Option<&'l Context<D, S, T, ST, SYM, VS, VT>>,
}

impl<D, S, T, ST, SYM, VS, VT> Identifiable for Model<'_, D, S, T, ST, SYM, VS, VT>
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
