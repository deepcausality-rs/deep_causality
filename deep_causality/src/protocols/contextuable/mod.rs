// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::*;

use crate::prelude::{ContextoidType, Identifiable, TimeScale};

pub trait Datable: Identifiable {}

pub trait Temporable<V>: Identifiable
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> &V;
}

pub trait Spatial<V>: Identifiable
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn x(&self) -> &V;
    fn y(&self) -> &V;
    fn z(&self) -> &V;
}

pub trait SpaceTemporal<V>: Identifiable + Spatial<V> + Temporable<V>
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn t(&self) -> &V; // returns 4th dimension, t
}

pub trait Contextuable<D, S, T, ST, V>: Identifiable
where
    D: Datable,
    S: Spatial<V>,
    ST: SpaceTemporal<V>,
    T: Temporable<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn vertex_type(&self) -> &ContextoidType<D, S, T, ST, V>;
}
