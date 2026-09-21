/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{ContextoidType, Datable};
use std::hash::Hash;

pub mod contextoid_type;
mod contextuable;
mod display;
mod identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Contextoid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    id: u64,
    vertex_type: ContextoidType<D, S, T, ST>,
}

impl<D, S, T, ST> Contextoid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    pub fn new(id: u64, vertex_type: ContextoidType<D, S, T, ST>) -> Self {
        Self { id, vertex_type }
    }
}
