// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use deep_causality_macros::{Constructor, Getters};

use crate::prelude::{Assumption, Causaloid, Context, Datable, Identifiable, SpaceTemporal, Spatial, Temporable};

#[derive(Getters, Constructor, Clone, Debug)]
pub struct Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporable + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    #[getter(name = model_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    author: &'l str,
    description: &'l str,
    assumptions: Option<&'l Vec<&'l Assumption>>,
    causaloid: &'l Causaloid<'l, D, S, T, ST>,
    context: Option<&'l Context<'l, D, S, T, ST>, >,
}

impl<'l, D, S, T, ST> Identifiable for Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporable + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn id(&self) -> u64 { self.id }
}
