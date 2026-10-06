/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric_signature;
mod recordable;
mod space_temporal;
mod space_temporal_interval;
mod spatial;
mod temporal;

use crate::TimeScale;
use deep_causality_algebra::RealField;

/// An event of Minkowski spacetime, in the coordinates `(t, x, y, z)` of an inertial frame.
///
/// Minkowski spacetime is the relativistic spacetime `(ℝ⁴, g_ab)` that is flat and geodesically
/// complete (Malament 2012, §2.1, pp. 127–128). This type has no curvature and no metric field:
/// the metric is the constant `η = diag(−1, +1, +1, +1)` on `x⁰ = ct, x¹ = x, x² = y, x³ = z`
/// (Carroll 1997, eqs. (1.5) and (1.8)). For an event that carries its own metric tensor, use
/// [`TangentSpacetime`](crate::TangentSpacetime).
///
/// The interval between two events is `s² = −(cΔt)² + Δx² + Δy² + Δz²` (Carroll 1997, eq. (1.3)),
/// computed by [`SpaceTemporalInterval::interval_squared`](crate::SpaceTemporalInterval). It is
/// negative for timelike, zero for null and positive for spacelike separation (Carroll 1997, §1,
/// after eq. (1.9)).
///
/// # Units and conventions
/// - `x`, `y`, `z` are in metres; `t` is counted in `time_scale` units and converted to seconds for
///   the interval. A scale that names no duration (`NoScale`, `Steps`, `Symbolic`) has no interval:
///   it is NaN.
/// - The signature is the east-coast (−, +, +, +), reported as `Metric::Lorentzian(4)`. The metric
///   crate's `Metric::Minkowski(4)` names the west-coast (+, −, −, −) form of the same metric.
///
/// # Coordinate index mapping
/// - `0 => t`
/// - `1 => x`
/// - `2 => y`
/// - `3 => z`
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// // Arguments: id, x, y, z, t, time scale.
/// let a = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
/// let b = MinkowskiSpacetime::new(2, 3.0, 4.0, 0.0, 0.0, TimeScale::Second);
///
/// // Simultaneous in this frame and 5 m apart: spacelike, s² = 25 m².
/// assert_eq!(a.interval_squared(&b), 25.0);
/// assert_eq!(*b.coordinate(1).unwrap(), 3.0);
/// ```
///
/// # References
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §2.1, pp. 119 and 127–128.
/// - Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019, ch. 1.
///   Copy: `papers/carroll_1997_lecture_notes_on_general_relativity_arXiv_gr-qc_9712019.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct MinkowskiSpacetime<R>
where
    R: RealField,
{
    id: ContextoidId,
    /// Coordinate time, in `time_scale` units.
    t: R,
    /// Position in metres.
    x: R,
    y: R,
    z: R,
    /// The unit `t` is counted in.
    time_scale: TimeScale,
}

impl<R: RealField> MinkowskiSpacetime<R> {
    /// An event at `(t, x, y, z)`. The arguments are given as `id, x, y, z, t, time_scale`.
    pub fn new(id: ContextoidId, x: R, y: R, z: R, t: R, time_scale: TimeScale) -> Self {
        Self {
            id,
            t,
            x,
            y,
            z,
            time_scale,
        }
    }
}
