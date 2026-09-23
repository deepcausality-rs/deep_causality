/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporal};
use ultragraph::UltraGraphWeighted;

/// One extra context: its name, its graph, and whether its identifier is the store's.
///
/// A stored extra context is a container referenced by identifier and name, so the name is kept
/// beside the graph it belongs to. An extra restored from a snapshot or attached by a store event
/// is `stored`; one created through `extra_ctx_add_new` or `extra_ctx_add_new_with_id` is local,
/// and no store event naming a container reaches it.
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
    pub(super) stored: bool,
}

impl<D, S, T, ST> ExtraContext<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// A local extra context.
    pub(super) fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            graph: UltraGraphWeighted::with_capacity(capacity, None),
            stored: false,
        }
    }

    /// An extra context whose identifier is the store's.
    pub(super) fn stored(name: &str, capacity: usize) -> Self {
        Self {
            stored: true,
            ..Self::new(name, capacity)
        }
    }
}
