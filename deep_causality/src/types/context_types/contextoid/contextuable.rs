// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Contextoid, ContextoidType, Contextuable, Datable, SpaceTemporal, Spatial, Temporable};

impl<D, S, T, ST> Contextuable<D, S, T, ST> for Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporable,
        ST: SpaceTemporal,
{
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST> {
        &self.vertex_type
    }
}
