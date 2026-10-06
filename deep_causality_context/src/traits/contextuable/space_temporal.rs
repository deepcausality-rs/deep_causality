/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::contextuable::coordinate::Coordinate;
use crate::traits::contextuable::metric_signature::MetricSignature;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;
use deep_causality_num::{FromPrimitive, lift};

/// An event of a spacetime: a node with a position and a time.
///
/// Every four-dimensional spacetime of this crate indexes its coordinates
/// `0 => t, 1 => x, 2 => y, 3 => z`, the order of Carroll 1997, eq. (1.5), with `t` at index 0
/// where Carroll places `x⁰ = ct`. `t()` is the time coordinate, counted in the unit
/// [`Temporal::time_scale`] names. What the time means is set by the geometry: absolute time in a
/// classical spacetime, the coordinate time of a frame in a relativistic one. [`MetricSignature`]
/// reports which. [`NoSpaceTime`](crate::NoSpaceTime) has zero coordinates, and its `t()` is `()`.
pub trait SpaceTemporal: Identifiable + Spatial + Temporal + MetricSignature {
    /// The time coordinate, coordinate 0.
    fn t(&self) -> &Self::TimeUnit;
}

/// The interval between two events of Minkowski spacetime,
///
/// ```text
/// s² = −(cΔt)² + Δx² + Δy² + Δz²
/// ```
///
/// with `t` in seconds, positions in metres and `c = 299 792 458 m/s` (Carroll 1997, eq. (1.3),
/// in the east-coast convention of eq. (1.8)). The interval is negative between timelike
/// separated events, zero between null separated ones and positive between spacelike separated
/// ones (Carroll 1997, §1, after eq. (1.9)). An implementor whose metric is not Minkowski's
/// overrides [`interval_squared`](Self::interval_squared), as
/// [`TangentSpacetime`](crate::TangentSpacetime) does.
///
/// # Required methods
/// - `time()`: the time coordinate in seconds, or NaN when the node's time names no duration
/// - `position()`: `[x, y, z]` in metres
///
/// # References
/// - Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019, ch. 1.
///   Copy: `papers/carroll_1997_lecture_notes_on_general_relativity_arXiv_gr-qc_9712019.pdf`.
///
/// # The scalar
/// The interval is measured in the same scalar the type's coordinates are, so this trait reads it
/// from [`Coordinate::Coord`] rather than declaring one of its own. A second associated type would
/// let a spacetime report its position in one scalar and its interval in another.
pub trait SpaceTemporalInterval: Coordinate
where
    Self::Coord: RealField + FromPrimitive,
{
    /// The time coordinate in seconds, or NaN when the node's time names no duration.
    fn time(&self) -> Self::Coord;

    /// The position `[x, y, z]` in metres.
    fn position(&self) -> [Self::Coord; 3];

    /// `s² = −(cΔt)² + Δx² + Δy² + Δz²`, in square metres.
    fn interval_squared(&self, other: &Self) -> Self::Coord {
        let c: Self::Coord = lift(299_792_458.0); // Speed of light (m/s)

        let dt = self.time() - other.time();
        let [x1, y1, z1] = self.position();
        let [x2, y2, z2] = other.position();

        let dx = x1 - x2;
        let dy = y1 - y2;
        let dz = z1 - z2;

        let c_dt = c * dt;
        -(c_dt * c_dt) + dx * dx + dy * dy + dz * dz
    }
}
