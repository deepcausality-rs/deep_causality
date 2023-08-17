// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ultragraph::prelude::*;

use crate::prelude::{
    ContextIndexError, Contextoid, ContextuableGraph, Datable,
    RelationKind, SpaceTemporal, Spatial, Temporal,
};

mod debug;
mod identifiable;
mod contextuable_graph;

pub struct Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    id: u64,
    name: &'l str,
    graph: UltraGraph<Contextoid<D, S, T, ST>>,
}


impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    /// Creates a new context with the given node capacity.
    pub fn with_capacity(
        id: u64,
        name: &'l str,
        capacity: usize,
    )
        -> Self
    {
        Self {
            id,
            name,
            graph: ultragraph::new_with_matrix_storage(capacity),
        }
    }

    /// Returns the name of the context.
    pub fn name(&self) -> &str {
        self.name
    }
}
