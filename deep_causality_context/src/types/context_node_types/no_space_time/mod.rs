/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;
use deep_causality_algebra::RealField;

mod coordinate;
mod display;
mod identifiable;
mod metric_signature;
mod space_temporal;
mod spatial;
mod temporal;

/// The absence of a spatial extent, as a type.
///
/// A `Context` names a spatial and a spacetime type whether or not its graph holds either, so a
/// context with a clock and no position still has two slots to fill. This fills them without
/// inventing a type the graph never holds, and says so in the context's own signature.
///
/// Measured usage makes this the ordinary case rather than an escape hatch. Across the workspace
/// the contextoid variants are used Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5, so
/// most contexts hold a root, some data and a clock, and nothing spatial at all.
///
/// # It is zero-sized
///
/// `R` is carried in a [`PhantomData`] so that this type can sit at a frame's scalar without
/// costing anything. `size_of::<NoSpaceTime<R>>()` is 0 for every `R`.
///
/// # What its trait impls say
///
/// The coordinate system has zero dimensions, so every index is out of bounds and
/// [`Coordinate::coordinate`](crate::Coordinate::coordinate) always returns an error. That is why
/// `Coord` can be `R` while nothing of type `R` is ever stored: the value is never produced.
///
/// The time unit is `()` rather than `R`. A frame with no spatial part still keeps a real clock in
/// its `Time` member; what this type stands in for is the *spacetime* position, which such a
/// context does not have, so there is no time coordinate here either.
///
/// # Example
/// ```
/// use deep_causality_context::{Coordinate, NoSpaceTime};
///
/// let empty: NoSpaceTime<f64> = NoSpaceTime::new();
///
/// assert_eq!(size_of::<NoSpaceTime<f64>>(), 0);
/// assert_eq!(empty.dimension(), 0);
/// assert!(empty.coordinate(0).is_err());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NoSpaceTime<R>
where
    R: RealField,
{
    scalar: PhantomData<R>,
}

impl<R> NoSpaceTime<R>
where
    R: RealField,
{
    pub fn new() -> Self {
        Self {
            scalar: PhantomData,
        }
    }
}
