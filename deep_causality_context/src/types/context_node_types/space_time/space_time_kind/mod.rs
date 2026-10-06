/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::errors::IndexError;
use crate::{
    Coordinate, GalileanSpacetime, MetricSignature, MinkowskiSpacetime, NewtonianSpacetime,
    SpaceTemporal, Spatial, TangentSpacetime, Temporal, TimeScale,
};
use core::fmt::Formatter;
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;

mod recordable;

/// One of the spacetime types a context can hold, so that a single spacetime slot can carry
/// events of different geometries.
///
/// # Variants
/// - `Galilean`: an event of Galilean spacetime; spatial distance only between simultaneous
///   events.
/// - `Newtonian`: an event of Newtonian spacetime, in coordinates at rest in absolute space.
/// - `Minkowski`: an event of flat Minkowski spacetime, in an inertial frame.
/// - `Tangent`: an event of a relativistic spacetime with its tangent vector and the metric
///   tensor there.
///
/// Every variant indexes its coordinates `0 => t, 1 => x, 2 => y, 3 => z`, and each reports its
/// own signature through [`MetricSignature`], so a context holding classical and relativistic
/// events answers correctly for each.
///
/// # Example
/// ```rust
/// use deep_causality_context::*;
///
/// let event = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 1.0, TimeScale::Second);
/// let spacetime = SpaceTimeKind::Newtonian(event);
///
/// assert_eq!(spacetime.dimension(), 4);
/// assert_eq!(spacetime.time_unit(), 1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceTimeKind<R>
where
    R: RealField,
{
    /// An event of Galilean spacetime.
    Galilean(GalileanSpacetime<R>),
    /// An event of Newtonian spacetime.
    Newtonian(NewtonianSpacetime<R>),
    /// An event of flat Minkowski spacetime.
    Minkowski(MinkowskiSpacetime<R>),
    /// An event of a relativistic spacetime with the metric tensor there.
    Tangent(TangentSpacetime<R>),
}

impl<R: RealField> Coordinate for SpaceTimeKind<R> {
    type Coord = R;

    fn dimension(&self) -> usize {
        match self {
            SpaceTimeKind::Galilean(s) => s.dimension(),
            SpaceTimeKind::Newtonian(s) => s.dimension(),
            SpaceTimeKind::Minkowski(s) => s.dimension(),
            SpaceTimeKind::Tangent(s) => s.dimension(),
        }
    }

    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        match self {
            SpaceTimeKind::Galilean(s) => s.coordinate(index),
            SpaceTimeKind::Newtonian(s) => s.coordinate(index),
            SpaceTimeKind::Minkowski(s) => s.coordinate(index),
            SpaceTimeKind::Tangent(s) => s.coordinate(index),
        }
    }
}

impl<R: RealField> Identifiable for SpaceTimeKind<R> {
    fn id(&self) -> ContextoidId {
        match self {
            SpaceTimeKind::Galilean(s) => s.id(),
            SpaceTimeKind::Newtonian(s) => s.id(),
            SpaceTimeKind::Minkowski(s) => s.id(),
            SpaceTimeKind::Tangent(s) => s.id(),
        }
    }
}

/// Forwards to the variant in hand. This is the whole point of asking the node: a context on
/// this enum holds nodes in more than one signature, and each answers for itself.
impl<R: RealField> MetricSignature for SpaceTimeKind<R> {
    fn metric(&self) -> deep_causality_metric::Metric {
        match self {
            SpaceTimeKind::Galilean(s) => s.metric(),
            SpaceTimeKind::Newtonian(s) => s.metric(),
            SpaceTimeKind::Minkowski(s) => s.metric(),
            SpaceTimeKind::Tangent(s) => s.metric(),
        }
    }
}

impl<R: RealField> Spatial for SpaceTimeKind<R> {}

impl<R: RealField> Temporal for SpaceTimeKind<R> {
    type TimeUnit = R;

    fn time_scale(&self) -> TimeScale {
        match self {
            SpaceTimeKind::Galilean(s) => s.time_scale(),
            SpaceTimeKind::Newtonian(s) => s.time_scale(),
            SpaceTimeKind::Minkowski(s) => s.time_scale(),
            SpaceTimeKind::Tangent(s) => s.time_scale(),
        }
    }

    fn time_unit(&self) -> R {
        match self {
            SpaceTimeKind::Galilean(s) => s.time_unit(),
            SpaceTimeKind::Newtonian(s) => s.time_unit(),
            SpaceTimeKind::Minkowski(s) => s.time_unit(),
            SpaceTimeKind::Tangent(s) => s.time_unit(),
        }
    }
}

impl<R: RealField> SpaceTemporal for SpaceTimeKind<R> {
    fn t(&self) -> &R {
        match self {
            SpaceTimeKind::Galilean(s) => s.t(),
            SpaceTimeKind::Newtonian(s) => s.t(),
            SpaceTimeKind::Minkowski(s) => s.t(),
            SpaceTimeKind::Tangent(s) => s.t(),
        }
    }
}

impl<R: RealField + core::fmt::Display> core::fmt::Display for SpaceTimeKind<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SpaceTimeKind::Galilean(s) => s.fmt(f),
            SpaceTimeKind::Newtonian(s) => s.fmt(f),
            SpaceTimeKind::Minkowski(s) => s.fmt(f),
            SpaceTimeKind::Tangent(s) => s.fmt(f),
        }
    }
}
