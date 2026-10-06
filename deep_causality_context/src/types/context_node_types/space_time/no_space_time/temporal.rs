/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{NoSpaceTime, Temporal, TimeScale};
use deep_causality_algebra::RealField;

impl<R: RealField> Temporal for NoSpaceTime<R> {
    /// The unit type. This stands in for a spacetime position, and a context with no spatial
    /// extent has no spacetime time coordinate either. Its clock lives in the frame's `Time`
    /// member, which is a separate type.
    type TimeUnit = ();

    fn time_scale(&self) -> TimeScale {
        TimeScale::NoScale
    }

    fn time_unit(&self) {}
}
