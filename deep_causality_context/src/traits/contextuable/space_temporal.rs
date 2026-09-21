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

/// Combines spatial and temporal semantics into a 4D spacetime model.
///
/// This is ideal for modeling causal entities that exist at a particular
/// **spatial location** and **point in time**. The `t()` method supplements
/// the coordinate system with a direct accessor for the temporal axis.
///
/// This trait enables compatibility with:
/// - Newtonian and Einsteinian physics
/// - Sensor frames
/// - 4D event graphs
///
/// # Note
/// The actual meaning of `t()` depends on the context—e.g., wall clock time,
/// simulation ticks, or a relativistic coordinate frame.
pub trait SpaceTemporal: Identifiable + Spatial + Temporal + MetricSignature {
    /// Returns the value associated with the temporal (4th) dimension.
    fn t(&self) -> &Self::TimeUnit;
}

/// Trait for spacetime types that support Minkowski-style interval calculations.
///
/// This trait enables causal reasoning in spacetime-aware systems using the Minkowski
/// metric from special relativity:
///
/// ```text
/// s² = -c²·Δt² + Δx² + Δy² + Δz²
/// ```
///
/// This interval:
/// - Is negative for **time-like** separations (causally connected)
/// - Is zero for **light-like** (null) paths (on the light cone)
/// - Is positive for **space-like** separations (no causal connection)
///
/// The default implementation assumes:
/// - Time is in **seconds**
/// - Space is in **meters**
/// - Speed of light `c = 299_792_458 m/s`
///
/// # Required Methods
/// - `time()`: Returns the scalar time coordinate in seconds
/// - `position()`: Returns the spatial coordinates `[x, y, z]` in meters
///
/// # Default Method
/// - `interval_squared(&self, &Self) -> Self::Coord`: Computes the squared interval between two
///   events
///
/// # The scalar
/// The interval is measured in the same scalar the type's coordinates are, so this trait reads it
/// from [`Coordinate::Coord`] rather than declaring one of its own. A second associated type would
/// let a spacetime report its position in one scalar and its interval in another.
pub trait SpaceTemporalInterval: Coordinate
where
    Self::Coord: RealField + FromPrimitive,
{
    /// Returns the time coordinate in **seconds**.
    fn time(&self) -> Self::Coord;

    /// Returns the spatial coordinates `[x, y, z]` in **meters**.
    fn position(&self) -> [Self::Coord; 3];

    /// Computes the squared Minkowski interval between `self` and `other`.
    ///
    /// ```text
    /// s² = -c²·Δt² + Δx² + Δy² + Δz²
    /// ```
    /// where `c = 299_792_458 m/s`.
    ///
    /// Negative `s²` indicates time-like separation,
    /// zero indicates light-like (null),
    /// and positive indicates space-like.
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
