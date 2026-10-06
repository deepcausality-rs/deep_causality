/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
mod adjustable;
mod display;
mod identifiable;
mod recordable;
mod scalar_projector;
mod temporable;

use crate::{TimeKind, TimeScale};
use deep_causality_algebra::RealField;

/// The absolute time of a classical spacetime: an instant, as a real number counted in
/// `time_scale` units.
///
/// In Galilean spacetime, and in every classical spacetime with more structure, the duration
/// between two events is a linear map `t` on the arrows between them; events it assigns zero are
/// simultaneous, and the duration is the same for every observer (Weatherall 2021, §3). Malament
/// states the same structure as the temporal metric `t_ab` of signature (1, 0, 0, 0) of a
/// classical spacetime (Malament 2012, §4.1). The difference of two `NewtonianTime` values in one
/// scale is that duration.
///
/// Galilean spacetime is an affine space of events (Weatherall 2021, §3), so no instant is
/// distinguished and the zero of the scale is a choice. [`Adjustable`](crate::Adjustable) accepts
/// any finite instant, zero and negative included, and refuses NaN and infinity.
///
/// The coordinate time of an inertial frame of Minkowski spacetime is a different quantity: it
/// depends on the frame. That is [`MinkowskiTime`](crate::MinkowskiTime).
///
/// # Example
/// ```rust
/// use deep_causality_context::{NewtonianTime, Identifiable, Temporal, TimeScale};
///
/// let t = NewtonianTime::new(42, TimeScale::Second, 12.5);
///
/// assert_eq!(t.id(), 42);
/// assert_eq!(t.time_scale(), TimeScale::Second);
/// assert_eq!(t.time_unit(), 12.5);
/// ```
///
/// # References
/// - Weatherall, J. O. (2021). Classical Spacetime Structure. In E. Knox & A. Wilson (Eds.),
///   *The Routledge Companion to Philosophy of Physics*, pp. 33–45. Routledge. arXiv:1707.05887,
///   §3. Copy: `papers/weatherall_2017_classical_spacetime_structure_arXiv_1707.05887.pdf`.
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §4.1, pp. 249–250.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NewtonianTime<R>
where
    R: RealField,
{
    /// Unique numeric identifier for the time instance.
    id: ContextoidId,

    /// The unit `time_unit` is counted in.
    time_scale: TimeScale,

    /// The instant, in `time_scale` units.
    time_unit: R,
}

impl<R: RealField> NewtonianTime<R> {
    pub fn new(id: ContextoidId, time_scale: TimeScale, time_unit: R) -> Self {
        Self {
            id,
            time_scale,
            time_unit,
        }
    }
}

impl<R: RealField> From<NewtonianTime<R>> for TimeKind<R> {
    fn from(t: NewtonianTime<R>) -> Self {
        TimeKind::Newtonian(t)
    }
}
