/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};

/// `t`, counted in `scale` units, in seconds; `None` for a scale that names no duration.
///
/// `Month`, `Quarter` and `Year` are means of the Gregorian calendar, whose year is 365.2425 days:
/// a year is 31 556 952 s, a month a twelfth of it and a quarter three months. `NoScale`, `Steps`
/// and `Symbolic` count without a unit of time.
pub(crate) fn seconds<R: RealField + FromPrimitive>(t: R, scale: TimeScale) -> Option<R> {
    match scale {
        TimeScale::Nanoseconds => Some(t / lift(1_000_000_000.0)),
        TimeScale::Microseconds => Some(t / lift(1_000_000.0)),
        TimeScale::Millisecond => Some(t / lift(1_000.0)),
        TimeScale::Second => Some(t),
        TimeScale::Minute => Some(t * lift(60.0)),
        TimeScale::Hour => Some(t * lift(3_600.0)),
        TimeScale::Day => Some(t * lift(86_400.0)),
        TimeScale::Week => Some(t * lift(604_800.0)),
        TimeScale::Month => Some(t * lift(2_629_746.0)),
        TimeScale::Quarter => Some(t * lift(7_889_238.0)),
        TimeScale::Year => Some(t * lift(31_556_952.0)),
        TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => None,
    }
}
