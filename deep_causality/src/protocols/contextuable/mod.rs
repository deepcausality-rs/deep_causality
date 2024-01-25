// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::*;

use crate::prelude::{ContextoidType, Identifiable, TimeScale};

pub trait Datable: Identifiable {}

/// Trait for types that have temporal properties.
///
/// V: Numeric type for time unit value
///
/// Requires:
/// - Identifiable: Has a unique ID
/// - V implements math ops: Add, Sub, Mul
///
/// Provides:
/// - time_scale(): Get the time scale (e.g. seconds, minutes)
/// - time_unit(): Get the time unit value for this item
///
pub trait Temporable<V>: Identifiable
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> &V;
}

/// Trait for types that have spatial properties.
///
/// V: Numeric type for spatial unit values
///
/// Requires:
/// - Identifiable: Has a unique ID
/// - V implements math ops: Add, Sub, Mul
///
/// Provides:
/// - x(): Get x spatial dimension value
/// - y(): Get y spatial dimension value
/// - z(): Get z spatial dimension value
///
pub trait Spatial<V>: Identifiable
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn x(&self) -> &V;
    fn y(&self) -> &V;
    fn z(&self) -> &V;
}

/// Trait for types with spatial and temporal properties.
///
/// V: Numeric type for dimension values
///
/// Requires:
/// - Identifiable: Has unique ID
/// - Spatial: Provides x, y, z spatial dims
/// - Temporable: Provides time scale and unit
/// - V implements math ops: Add, Sub, Mul
///
/// Provides:
/// - t(): Get value for 4th (temporal) dimension
///
pub trait SpaceTemporal<V>: Identifiable + Spatial<V> + Temporable<V>
where
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn t(&self) -> &V; // returns 4th dimension, t
}

/// Trait for context-aware types with spatial, temporal, and datable properties.
///
/// D: Datable trait object
/// S: Spatial trait object
/// T: Temporable trait object
/// ST: SpaceTemporal trait object
/// V: Numeric type for dimension values
///
/// Requires:
/// - Identifiable: Has unique ID
/// - D, S, T, ST implement respective traits
/// - V implements math ops: Add, Sub, Mul
///
/// Provides:
/// - vertex_type(): Get the vertex type (D, S, T, ST)
///
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
