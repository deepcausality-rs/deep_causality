/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DiscreteTime, Temporal, TimeScale};

impl Temporal for DiscreteTime {
    type TimeUnit = u64;
    fn time_scale(&self) -> TimeScale {
        self.tick_scale
    }

    fn time_unit(&self) -> u64 {
        self.tick_unit
    }
}
