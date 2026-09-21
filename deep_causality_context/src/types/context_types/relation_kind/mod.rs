/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Debug, Display};

/// The kind of relation an edge in a `Context` carries.
///
/// The `Default` exists to satisfy the backing graph's `W: Clone + Default` weight bound. Edge
/// construction always supplies a relation, so the default never reaches an edge through this
/// crate's API. `Datial` carries it because a data relation is the least specific of the four, so
/// an accidental appearance claims the least.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum RelationKind {
    #[default]
    Datial,
    Temporal,
    Spatial,
    SpaceTemporal,
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
