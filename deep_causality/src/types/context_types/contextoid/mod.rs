// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{ContextoidType, Datable, SpaceTemporal, Spatial, Temporal};

mod identifiable;
mod display;
mod contextuable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST>,
}

impl<D, S, T, ST> Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST>) -> Self
    {
        Self { id, vertex_type }
    }
}

