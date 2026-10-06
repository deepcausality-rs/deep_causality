/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;
use deep_causality_algebra::RealField;

mod coordinate;
mod display;
mod identifiable;
mod recordable;
mod spatial;

/// The absence of a spatial position, as a type: it fills the spatial slot of a `Context` whose
/// graph holds no spatial node.
///
/// A `Context` names a spatial, a temporal and a spacetime type whether or not its graph holds
/// nodes of each. `NoSpace` fills the spatial slot and [`NoSpaceTime`](crate::NoSpaceTime) the
/// spacetime slot, so the context's signature names what is absent in each:
///
/// ```
/// use deep_causality_context::{Context, Data, DiscreteTime, NoSpace, NoSpaceTime};
///
/// type Clockwork = Context<Data<f64>, NoSpace<f64>, DiscreteTime, NoSpaceTime<f64>>;
///
/// let context: Clockwork = Context::with_capacity(1, "clock only", 4);
/// assert_eq!(context.name(), "clock only");
/// ```
///
/// # It is zero-sized
///
/// `R` is carried in a [`PhantomData`] so that this type can sit at a frame's scalar without
/// costing anything. `size_of::<NoSpace<R>>()` is 0 for every `R`.
///
/// # What its trait impls say
///
/// The coordinate system has zero dimensions, so every index is out of bounds and
/// [`Coordinate::coordinate`](crate::Coordinate::coordinate) always returns an error. That is why
/// `Coord` can be `R` while nothing of type `R` is ever stored: the value is never produced. It
/// has no record: writing one is `Unrecordable`, reading one is `WrongVariant`.
///
/// # Example
/// ```
/// use deep_causality_context::{Coordinate, NoSpace};
///
/// let empty: NoSpace<f64> = NoSpace::new();
///
/// assert_eq!(size_of::<NoSpace<f64>>(), 0);
/// assert_eq!(empty.dimension(), 0);
/// assert!(empty.coordinate(0).is_err());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NoSpace<R>
where
    R: RealField,
{
    scalar: PhantomData<R>,
}

impl<R> NoSpace<R>
where
    R: RealField,
{
    pub fn new() -> Self {
        Self {
            scalar: PhantomData,
        }
    }
}
