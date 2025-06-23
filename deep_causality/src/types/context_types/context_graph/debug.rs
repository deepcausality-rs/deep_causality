// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Context, Datable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::traits::contextuable_graph::ContextuableGraph;

impl<D, S, T, ST, SYM, V> Context<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    fn format(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context: id: {}, name: {}, node_count: {}, edge_count: {}",
            self.id,
            self.name,
            self.node_count(),
            self.edge_count(),
        )
    }
}

impl<D, S, T, ST, SYM, V> Debug for Context<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<D, S, T, ST, SYM, V> Display for Context<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}
