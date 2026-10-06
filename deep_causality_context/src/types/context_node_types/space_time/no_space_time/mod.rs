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
mod recordable;
mod space_temporal;
mod spatial;
mod temporal;

/// The absence of a spacetime position, as a type: it fills the spacetime slot of a `Context`
/// whose graph holds no spacetime node.
///
/// A `Context` names a spatial, a temporal and a spacetime type whether or not its graph holds
/// nodes of each. `NoSpaceTime` fills the spacetime slot, beside [`NoSpace`](crate::NoSpace) for
/// the spatial slot and [`NoTime`](crate::NoTime) for the temporal one, so the context's own
/// signature says what it does not hold. Because `SpaceTemporal` requires `Spatial` and
/// `Temporal`, this type also satisfies the spatial and temporal slots.
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
/// The time unit is `()` rather than `R`: what this type stands in for is a spacetime position,
/// which the context does not have, so there is no time coordinate here either. A context's clock
/// lives in its temporal slot.
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
