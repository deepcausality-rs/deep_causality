/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianSpacetime, SpaceTemporalInterval, TimeScale};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};

impl<R: RealField + FromPrimitive> SpaceTemporalInterval for LorentzianSpacetime<R> {
    fn time(&self) -> R {
        // Convert time to seconds based on time_scale
        // SpaceTemporalInterval trait contract requires time in seconds
        match self.time_scale {
            TimeScale::Nanoseconds => self.t / lift(1_000_000_000.0),
            TimeScale::Microseconds => self.t / lift(1_000_000.0),
            TimeScale::Millisecond => self.t / lift(1_000.0),
            TimeScale::Second => self.t,
            TimeScale::Minute => self.t * lift(60.0),
            TimeScale::Hour => self.t * lift(3_600.0),
            TimeScale::Day => self.t * lift(86_400.0),
            TimeScale::Week => self.t * lift(604_800.0),
            TimeScale::Month => self.t * lift(2_629_746.0), // Average month (365.2425 days / 12)
            TimeScale::Quarter => self.t * lift(7_889_238.0), // 3 months
            TimeScale::Year => self.t * lift(31_556_952.0), // Gregorian year (365.2425 days)
            // For non-physical time scales, return raw value
            TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => self.t,
        }
    }
    fn position(&self) -> [R; 3] {
        [self.x, self.y, self.z]
    }
    // No need to override `interval_squared()` unless you want a custom metric for curved spacetime
}
