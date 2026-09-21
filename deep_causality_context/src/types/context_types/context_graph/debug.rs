/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Debug, Display, Formatter};

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::traits::contextuable_graph::ContextuableGraph;
use crate::{Context, Datable};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn format(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context: id: {}, name: {}, node_count: {}, edge_count: {}",
            self.id,
            self.name,
            self.number_of_nodes(),
            self.number_of_edges(),
        )
    }
}

impl<D, S, T, ST> Debug for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<D, S, T, ST> Display for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}
