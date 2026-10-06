/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{NoTime, Temporal, TimeScale};

impl Temporal for NoTime {
    /// The unit type: there is no instant to report.
    type TimeUnit = ();

    fn time_scale(&self) -> TimeScale {
        TimeScale::NoScale
    }

    fn time_unit(&self) {}
}
