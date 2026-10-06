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
mod metric;
mod metric_signature;
mod recordable;
mod space_temporal;
mod spatial;
mod temporal;

use crate::TimeScale;
use deep_causality_algebra::RealField;

/// An event of Newtonian spacetime, in coordinates `(t, x, y, z)` at rest in its absolute space.
///
/// Newtonian spacetime is Galilean spacetime with one further structure: a privileged arrow `ξ`
/// of unit temporal length, Newton's absolute space. Two events are at the same place, at
/// different times, exactly when the arrow between them is a multiple of `ξ` (Weatherall 2021,
/// §4; Stein 1967). Galilean spacetime itself is a four-dimensional affine space with a linear
/// duration `t` and a Euclidean metric on each set of simultaneous events (Weatherall 2021, §3);
/// see [`GalileanSpacetime`](crate::GalileanSpacetime).
///
/// The coordinates of this type are at rest with `ξ`: `t` is the absolute time and `x, y, z` are
/// Cartesian positions in absolute space, in metres. Because places persist through time, the
/// spatial distance between any two events is defined, including events at different times. It
/// is the norm `√(ĥ_ab Δxᵃ Δxᵇ)` of the spatial metric `ĥ_ab` that `ξ` determines
/// (Malament 2012, §4.1, Proposition 4.1.2), and in these coordinates
/// `ĥ_ab = diag(0, 1, 1, 1)`: [`Distance`](crate::Distance) is the Euclidean norm of
/// `(Δx, Δy, Δz)` and does not depend on `Δt`. The duration between the events is `Δt`.
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
/// let a = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
/// let b = NewtonianSpacetime::new(2, 3.0, 4.0, 0.0, 60.0, TimeScale::Second);
///
/// // A minute apart, and 5 m apart in absolute space.
/// assert_eq!(a.distance(&b), 5.0);
/// assert_eq!(*b.coordinate(0).unwrap(), 60.0);
/// ```
///
/// # References
/// - Weatherall, J. O. (2021). Classical Spacetime Structure. In E. Knox & A. Wilson (Eds.),
///   *The Routledge Companion to Philosophy of Physics*, pp. 33–45. Routledge. arXiv:1707.05887,
///   §3 and §4. Copy: `papers/weatherall_2017_classical_spacetime_structure_arXiv_1707.05887.pdf`.
/// - Stein, H. (1967). Newtonian space-time. *The Texas Quarterly* 10, 174–200.
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §4.1, pp. 249–255.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NewtonianSpacetime<R>
where
    R: RealField,
{
    /// Unique numeric ID for this context
    id: ContextoidId,
    /// Absolute time, in `time_scale` units.
    t: R,
    /// Position in absolute space, in metres.
    x: R,
    y: R,
    z: R,
    /// The unit `t` is counted in.
    time_scale: TimeScale,
}

impl<R: RealField> NewtonianSpacetime<R> {
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
