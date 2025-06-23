// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{
    ContextoidType, Datable, Identifiable, SpaceTemporal, Spatial, Symbolic, Temporal,
};

pub mod coordinate;
pub mod datable;
pub mod metric;
pub mod metric_coordinate;
pub mod metric_tensor;
pub mod space_temporal;
pub mod spatial;
pub mod symbolic;
pub mod temporal;

/// Represents any entity that participates in a causal context graph.
///
/// This trait defines the unified interface over any entity that may be:
/// - A data node
/// - A spatial or temporal marker
/// - A symbolic atom
/// - A spacetime event
///
/// It is designed to **abstract over the underlying causal semantics**
/// while retaining compile-time type safety and minimal trait bounds.
///
/// # Type Parameters
/// - `D`: A [`Datable`] node (e.g., sensor reading, fact, entity)
/// - `S`: A [`Spatial`] node
/// - `T`: A [`Temporal`] node
/// - `ST`: A [`SpaceTemporal`] node (4D entity)
/// - `SYM`: A [`Symbolic`] node (logical/abstract)
/// - `V`: The numeric or symbolic coordinate type
///
/// # Design Note
/// This trait is the dispatch point for `ContextoidType`, allowing static or
/// dynamic graph traversal based on node kind. It intentionally generalizes
/// over all possible causal node roles.
pub trait Contextuable<D, S, T, ST, SYM, V>: Identifiable
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    /// Returns a reference to the type-erased node variant.
    ///
    /// Use this to determine the role of the current node (data, space, time, etc.)
    /// and then downcast or dispatch accordingly.
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST, SYM, V>;
}
