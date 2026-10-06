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
mod simultaneity;
mod space_temporal;
mod spatial;
mod temporal;

use crate::TimeScale;
use deep_causality_algebra::RealField;

/// An event of Galilean spacetime, in the coordinates `(t, x, y, z)` of an inertial frame.
///
/// Galilean spacetime is a four-dimensional affine space of events with two structures: a
/// non-vanishing linear functional `t` on the arrows between events, the duration, and on each set
/// of simultaneous events (those the duration assigns zero) a Euclidean metric
/// (Weatherall 2021, §3 and footnote 13). Malament states the same structure as a classical
/// spacetime with temporal metric `t_ab` of signature (1, 0, 0, 0) and spatial metric `h^ab` of
/// signature (0, 1, 1, 1) (Malament 2012, §4.1).
///
/// Galilean spacetime has no notion of spatial distance between events that are not
/// simultaneous (Weatherall 2021, §3, figure 2): which places at different times are "the same"
/// depends on the observer. [`Distance`](crate::Distance) is therefore defined only for two
/// events at the same instant, where it is the Euclidean norm of `(Δx, Δy, Δz)`; for any other
/// pair it is NaN. Adding a privileged rest frame, absolute space, gives Newtonian spacetime, in
/// which every pair has a distance: [`NewtonianSpacetime`](crate::NewtonianSpacetime).
///
/// The frame of the coordinates is any inertial frame; Galilean spacetime singles none out. `t`
/// is counted in `time_scale` units and `x, y, z` are in metres.
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
/// let a: GalileanSpacetime<f64> = GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 60.0, TimeScale::Second);
/// let b = GalileanSpacetime::new(2, 3.0, 4.0, 0.0, 1.0, TimeScale::Minute);
/// let later = GalileanSpacetime::new(3, 3.0, 4.0, 0.0, 61.0, TimeScale::Second);
///
/// // The same instant, so the distance is defined.
/// assert_eq!(a.distance(&b), 5.0);
/// // A second later: no distance in Galilean spacetime.
/// assert!(a.distance(&later).is_nan());
/// ```
///
/// # References
/// - Weatherall, J. O. (2021). Classical Spacetime Structure. In E. Knox & A. Wilson (Eds.),
///   *The Routledge Companion to Philosophy of Physics*, pp. 33–45. Routledge. arXiv:1707.05887,
///   §3. Copy: `papers/weatherall_2017_classical_spacetime_structure_arXiv_1707.05887.pdf`.
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §4.1, pp. 249–250.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GalileanSpacetime<R>
where
    R: RealField,
{
    id: ContextoidId,
    /// Time, in `time_scale` units.
    t: R,
    /// Position in metres, in the frame of the coordinates.
    x: R,
    y: R,
    z: R,
    /// The unit `t` is counted in.
    time_scale: TimeScale,
}

impl<R: RealField> GalileanSpacetime<R> {
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
