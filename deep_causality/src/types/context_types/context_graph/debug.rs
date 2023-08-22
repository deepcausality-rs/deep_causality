// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Context, ContextuableGraph, Datable, SpaceTemporal, Spatial, Temporable};

impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporable,
        ST: SpaceTemporal,
{
    fn format(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Context: id: {}, name: {}, node_count: {}, edge_count: {}",
               self.id,
               self.name,
               self.node_count(),
               self.edge_count(),
        )
    }
}

impl<'l, D, S, T, ST> Debug for Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporable,
        ST: SpaceTemporal,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<'l, D, S, T, ST> Display for Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporable,
        ST: SpaceTemporal,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}
