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
pub struct Contextoid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST, SYM, VS, VT>,
}

impl<D, S, T, ST, SYM, VS, VT> Contextoid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST, SYM, VS, VT>) -> Self {
        Self { id, vertex_type }
    }
}
