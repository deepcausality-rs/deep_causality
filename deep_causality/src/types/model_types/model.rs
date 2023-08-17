// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use crate::prelude::{Assumption, Causaloid, Context, Datable, SpaceTemporal, Spatial, Temporal};
use crate::protocols::identifiable::Identifiable;

#[derive(Clone, Copy)]
pub struct Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    id: u64,
    author: &'l str,
    description: &'l str,
    assumptions: Option<&'l Vec<&'l Assumption>>,
    causaloid: &'l Causaloid<'l, D, S, T, ST>,
    context: Option<&'l Context<'l, D, S, T, ST>,>,
}

impl<'l, D, S, T, ST> Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    pub fn new(
        id: u64,
        author: &'l str,
        description: &'l str,
        assumptions: Option<&'l Vec<&'l Assumption>>,
        causaloid: &'l Causaloid<'l, D, S, T, ST>,
        context: Option<&'l Context<'l, D, S, T, ST>>
    ) -> Self {
        Self {
            id,
            author,
            description,
            assumptions,
            causaloid,
            context
        }
    }
}

impl<'l, D, S, T, ST> Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    pub fn author(&self) -> &'l str {
        self.author
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn assumptions(&self) -> Option<&'l Vec<&'l Assumption>> {
        self.assumptions
    }
    pub fn causaloid(&self) -> &'l Causaloid<D, S, T, ST> {
        self.causaloid
    }
    pub fn context(&self) -> Option<&'l Context<'l, D, S, T, ST>> {
        self.context
    }
}

impl<'l, D, S, T, ST> Identifiable for Model<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn id(&self) -> u64 {
        self.id
    }
}