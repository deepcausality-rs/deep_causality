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
    fn add_extra_context(&mut self, capacity: usize, default: bool) -> usize;
    fn check_extra_context_exists(&self, idx: usize) -> bool;
    fn set_extra_default_context(&mut self, idx: usize) -> Result<(), ContextIndexError>;
    fn unset_extra_default_context(&mut self) -> Result<(), ContextIndexError>;
}
