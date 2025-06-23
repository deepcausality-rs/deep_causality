// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::hash::Hash;
use crate::prelude::{ContextoidType, Datable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

pub mod contextoid_type;
mod contextuable;
mod display;
mod identifiable;

#[derive(Debug, Copy, Clone, Hash)]
pub struct Contextoid<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST, SYM, V>,
}

impl<D, S, T, ST, SYM, V> Contextoid<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST, SYM, V>) -> Self {
        Self { id, vertex_type }
    }
}
