// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use std::ops::*;

use crate::prelude::{ContextoidType, Datable, SpaceTemporal, Spatial, Temporable};

pub mod contextoid_type;
mod contextuable;
mod display;
mod identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Contextoid<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST, V>,
}

impl<D, S, T, ST, V> Contextoid<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST, V>) -> Self {
        Self {
            id,
            vertex_type,
        }
    }
}
