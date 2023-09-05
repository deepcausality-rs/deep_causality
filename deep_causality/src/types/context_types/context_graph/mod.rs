// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::*;

use ultragraph::prelude::*;

use crate::prelude::{
    ContextIndexError, Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporable,
};
use crate::protocols::contextuable_graph::ContextuableGraph;

mod contextuable_graph;
mod debug;
mod identifiable;

pub struct Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    id: u64,
    name: &'l str,
    graph: UltraGraph<Contextoid<D, S, T, ST, V>>,
}

impl<'l, D, S, T, ST, V> Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    /// Creates a new context with the given node capacity.
    pub fn with_capacity(id: u64, name: &'l str, capacity: usize) -> Self {
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
