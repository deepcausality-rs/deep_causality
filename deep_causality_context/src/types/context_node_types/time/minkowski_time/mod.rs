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

/// The coordinate time `t` of an inertial frame of Minkowski spacetime, counted in `time_scale`
/// units.
///
/// An event of Minkowski spacetime has coordinates `(t, x, y, z)` in an inertial frame, with
/// `x⁰ = ct` (Carroll 1997, eq. (1.5)). Unlike the absolute time of a classical spacetime, `t`
/// depends on the frame: whether two events are simultaneous depends on the coordinates used
/// (Carroll 1997, §1, after eq. (1.4)). The frame-independent quantity is the interval, which
/// [`MinkowskiSpacetime`](crate::MinkowskiSpacetime) computes.
///
/// A translation `x^μ → x^μ + a^μ` leaves the interval unchanged (Carroll 1997, eq. (1.10)), so
/// the zero of `t` is a choice. [`Adjustable`](crate::Adjustable) accepts any finite coordinate
/// time, zero and negative included, and refuses NaN and infinity.
///
/// # Example
/// ```rust
/// use deep_causality_context::{Identifiable, MinkowskiTime, Temporal, TimeScale};
///
/// let t = MinkowskiTime::new(1, TimeScale::Second, core::f64::consts::E);
///
/// assert_eq!(t.id(), 1);
/// assert_eq!(t.time_scale(), TimeScale::Second);
/// ```
///
/// # References
/// - Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019, ch. 1.
///   Copy: `papers/carroll_1997_lecture_notes_on_general_relativity_arXiv_gr-qc_9712019.pdf`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MinkowskiTime<R>
where
    R: RealField,
{
    /// Unique numeric identifier for the time instance.
    id: ContextoidId,

    /// The unit `time_unit` is counted in.
    time_scale: TimeScale,

    /// The coordinate time, in `time_scale` units.
    time_unit: R,
}

impl<R: RealField> MinkowskiTime<R> {
    pub fn new(id: ContextoidId, time_scale: TimeScale, time_unit: R) -> Self {
        Self {
            id,
            time_scale,
            time_unit,
        }
    }
}

impl<R: RealField> From<MinkowskiTime<R>> for TimeKind<R> {
    fn from(t: MinkowskiTime<R>) -> Self {
        TimeKind::Minkowski(t)
    }
}
