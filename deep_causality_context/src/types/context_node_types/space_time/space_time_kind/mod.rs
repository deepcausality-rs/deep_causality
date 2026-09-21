/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::errors::IndexError;
use crate::{
    Coordinate, EuclideanSpacetime, LorentzianSpacetime, MetricSignature, SpaceTemporal, Spatial,
    TangentSpacetime, Temporal, TimeScale,
};
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;
use std::fmt::Formatter;

/// A polymorphic enum over supported spacetime context types.
///
/// `SpaceTimeKind` provides a unified abstraction over multiple spacetime representations.
/// It enables algorithms to generically operate over different mathematical models of
/// space and time without requiring monomorphic type coupling.
///
/// This type implements key traits such as [`Coordinate`], [`Temporal`], [`Spatial`], and
/// [`SpaceTemporal`] to enable high-level reasoning, measurement, and causal modeling in spacetime-aware systems.
///
/// # Supported Variants
///
/// - [`EuclideanSpacetime`]: Classical Newtonian model with separate space and time.
/// - [`LorentzianSpacetime`]: Supports pseudo-Riemannian geometry, used in general relativity.
/// - [`TangentSpacetime`]: Linear approximation of curved space at a point (i.e., tangent space).
///
/// # Examples
///
/// ```rust
/// use deep_causality_context::*;
///
/// let euclidean = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 1.0, TimeScale::Second);
/// let spacetime = SpaceTimeKind::Euclidean(euclidean);
///
/// assert_eq!(spacetime.dimension(), 4);
/// assert_eq!(spacetime.time_unit(), 1.0);
/// ```
///
/// # Trait Support
///
/// `SpaceTimeKind` implements:
///
/// - [`Identifiable`]: Unique ID for referencing entities.
/// - [`Coordinate`]: Spatial dimensionality and index access.
/// - [`Temporal`]: Temporal metadata and tick-based behavior.
/// - [`Spatial`]: Marker trait for spatial context.
/// - [`SpaceTemporal`]: Full space-time reasoning abstraction.
///
/// # Index Mapping
/// Coordinate indexing depends on the inner type variant. The most common mapping is:
/// - `0 => x`
/// - `1 => y`
/// - `2 => z`
/// - `3 => t`
///
/// # Notes
/// - This abstraction is ideal for heterogeneous systems that must support multiple
///   physical or geometric models simultaneously (e.g., causal simulation engines,
///   robotics frameworks, or time-aware decision systems).
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceTimeKind<R>
where
    R: RealField,
{
    /// Classical Newtonian spacetime (ℝ³ + time)
    Euclidean(EuclideanSpacetime<R>),
    /// General relativistic curved spacetime
    Lorentzian(LorentzianSpacetime<R>),
    /// Tangent space at a point, used for local linearization of curvature
    Tangent(TangentSpacetime<R>),
}

impl<R: RealField> Coordinate for SpaceTimeKind<R> {
    type Coord = R;
    fn dimension(&self) -> usize {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.dimension(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.dimension(),
            SpaceTimeKind::Tangent(tangent) => tangent.dimension(),
        }
    }

    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.coordinate(index),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.coordinate(index),
            SpaceTimeKind::Tangent(tangent) => tangent.coordinate(index),
        }
    }
}

impl<R: RealField> Identifiable for SpaceTimeKind<R> {
    fn id(&self) -> ContextoidId {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.id(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.id(),
            SpaceTimeKind::Tangent(tangent) => tangent.id(),
        }
    }
}

/// Forwards to the variant in hand. This is the whole point of asking the node: a context on
/// this enum holds nodes in more than one signature, and each answers for itself.
impl<R: RealField> MetricSignature for SpaceTimeKind<R> {
    fn metric(&self) -> deep_causality_metric::Metric {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.metric(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.metric(),
            SpaceTimeKind::Tangent(tangent) => tangent.metric(),
        }
    }
}

impl<R: RealField> Spatial for SpaceTimeKind<R> {}

impl<R: RealField> Temporal for SpaceTimeKind<R> {
    type TimeUnit = R;
    fn time_scale(&self) -> TimeScale {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.time_scale(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.time_scale(),
            SpaceTimeKind::Tangent(tangent) => tangent.time_scale(),
        }
    }

    fn time_unit(&self) -> R {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.time_unit(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.time_unit(),
            SpaceTimeKind::Tangent(tangent) => tangent.time_unit(),
        }
    }
}

impl<R: RealField> SpaceTemporal for SpaceTimeKind<R> {
    fn t(&self) -> &R {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.t(),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.t(),
            SpaceTimeKind::Tangent(tangent) => tangent.t(),
        }
    }
}

impl<R: RealField + std::fmt::Display> std::fmt::Display for SpaceTimeKind<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceTimeKind::Euclidean(euclidean) => euclidean.fmt(f),
            SpaceTimeKind::Lorentzian(lorentzian) => lorentzian.fmt(f),
            SpaceTimeKind::Tangent(tangent) => tangent.fmt(f),
        }
    }
}
