// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{Contextoid, ContextoidType, Contextuable, Datable, SpaceTemporal, Spatial, Temporal};

impl<D, S, T, ST> Contextuable<D, S, T, ST> for Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST> {
        &self.vertex_type
    }
}
