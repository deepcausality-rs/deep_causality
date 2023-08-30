// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use crate::prelude::{Contextoid, Datable, Identifiable, SpaceTemporal, Spatial, Temporable};

impl<D, S, T, ST, V> Identifiable for Contextoid<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn id(&self) -> u64 {
        self.id
    }
}
