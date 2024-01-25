use std::fmt::{Display, Formatter};
use std::hash::Hash;
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use crate::prelude::{Contextoid, Datable, SpaceTemporal, Spatial, Temporable};

impl<D, S, T, ST, V> Display for Contextoid<D, S, T, ST, V>
where
    D: Datable + Display,
    S: Spatial<V> + Display,
    T: Temporable<V> + Display,
    ST: SpaceTemporal<V> + Display,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contextoid ID: {} Type: {}", self.id, self.vertex_type)
    }
}
