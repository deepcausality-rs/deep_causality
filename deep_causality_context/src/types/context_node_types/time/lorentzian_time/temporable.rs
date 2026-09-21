/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianTime, Temporal, TimeScale};
use deep_causality_algebra::RealField;

impl<R: RealField> Temporal for LorentzianTime<R> {
    type TimeUnit = R;
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> R {
        self.time_unit
    }
}
