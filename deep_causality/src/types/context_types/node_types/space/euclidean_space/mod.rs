// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Debug};

use deep_causality_macros::Constructor;

mod display;
mod identifiable;
mod spatial;
mod coordinate;
mod metric;
mod getters;

/// A 3-dimensional spatial context represented in standard Euclidean coordinates (x, y, z).
///
/// This struct is used as the default implementation of a purely spatial context
/// in the DeepCausality framework. It supports coordinate access and distance
/// measurement based on Euclidean geometry.
///
/// # Fields
/// - `id`: A unique identifier for this spatial entity
/// - `coords`: A `[f64; 3]` array representing the spatial coordinates in 3D space
///
/// # Traits Implemented
/// - [`Identifiable`]
/// - [`Coordinate<f64>`]
/// - [`Metric`]
/// - [`Spatial<f64>`]
///
/// # Examples
/// ```
/// use deep_causality::prelude::*;
///
/// let space_a = EuclideanSpace::new(1, [1.0, 2.0, 3.0]);
/// let space_b = EuclideanSpace::new(2, [4.0, 6.0, 3.0]);
///
/// assert_eq!(space_a.dimension(), 3);
/// assert_eq!(space_a.coordinate(1), &2.0);
/// assert_eq!(space_a.distance(&space_b), 5.0);
/// ```
#[derive(Constructor, Debug, Clone, PartialEq)]
pub struct EuclideanSpace {
      id: u64,
      coords: [f64; 3],
}

