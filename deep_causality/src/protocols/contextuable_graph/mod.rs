// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::{Add, Mul, Sub};

use crate::errors::ContextIndexError;
use crate::prelude::{Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporable};

pub trait ContextuableGraph<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    ST: SpaceTemporal<V>,
    T: Temporable<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn add_node(&mut self, value: Contextoid<D, S, T, ST, V>) -> usize;
    fn contains_node(&self, index: usize) -> bool;
    fn get_node(&self, index: usize) -> Option<&Contextoid<D, S, T, ST, V>>;
    fn remove_node(&mut self, index: usize) -> Result<(), ContextIndexError>;
    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError>;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}

pub trait ExtendableContextuableGraph<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    ST: SpaceTemporal<V>,
    T: Temporable<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    // Creates a new context and returns the index of the new context.
    fn extra_ctx_add_new(&mut self, capacity: usize, default: bool) -> u64;
    fn extra_ctx_check_exists(&self, idx: u64) -> bool;
    fn extra_ctx_get_current_id(&self) -> u64;
    fn extra_ctx_set_current_id(&mut self, idx: u64) -> Result<(), ContextIndexError>;
    fn extra_ctx_unset_current_id(&mut self) -> Result<(), ContextIndexError>;

    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST, V>,
    ) -> Result<usize, ContextIndexError>;
    fn extra_ctx_contains_node(&self, index: usize) -> bool;
    fn extra_ctx_get_node(
        &self,
        index: usize,
    ) -> Result<&Contextoid<D, S, T, ST, V>, ContextIndexError>;
    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError>;
    fn extra_ctx_add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;
    fn extra_ctx_contains_edge(&self, a: usize, b: usize) -> bool;
    fn extra_ctx_remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError>;
    fn extra_ctx_size(&self) -> Result<usize, ContextIndexError>;
    fn extra_ctx_is_empty(&self) -> Result<bool, ContextIndexError>;
    fn extra_ctx_node_count(&self) -> Result<usize, ContextIndexError>;
    fn extra_ctx_edge_count(&self) -> Result<usize, ContextIndexError>;
}
