/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianSpacetime, Temporal, TimeScale};
use deep_causality_algebra::RealField;

impl<R: RealField> Temporal for LorentzianSpacetime<R> {
    type TimeUnit = R;
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> R {
        self.t
    }
}
