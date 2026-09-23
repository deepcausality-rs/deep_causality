/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporal};
use ultragraph::UltraGraphWeighted;

/// One extra context: its name and its graph.
///
/// A stored extra context is a container referenced by identifier and name, so the name is kept
/// beside the graph it belongs to.
#[derive(Clone)]
pub(super) struct ExtraContext<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    pub(super) name: String,
    pub(super) graph: UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>,
}

impl<D, S, T, ST> ExtraContext<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    pub(super) fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            graph: UltraGraphWeighted::with_capacity(capacity, None),
        }
    }
}
