/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianTime, Temporal, TimeScale};

impl Temporal for LorentzianTime {
    type TimeUnit = f64;
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> f64 {
        self.time_unit
    }
}
