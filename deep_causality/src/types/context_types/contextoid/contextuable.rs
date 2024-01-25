use std::hash::Hash;
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use crate::prelude::{
    Contextoid, ContextoidType, Contextuable, Datable, SpaceTemporal, Spatial, Temporable,
};

impl<D, S, T, ST, V> Contextuable<D, S, T, ST, V> for Contextoid<D, S, T, ST, V>
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
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST, V> {
        &self.vertex_type
    }
}
