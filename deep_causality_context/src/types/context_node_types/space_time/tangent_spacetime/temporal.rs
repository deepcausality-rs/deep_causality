/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TangentSpacetime, Temporal, TimeScale};
use deep_causality_algebra::RealField;

impl<R: RealField> Temporal for TangentSpacetime<R> {
    type TimeUnit = R;
    fn time_scale(&self) -> TimeScale {
        TimeScale::Second
    }
    fn time_unit(&self) -> R {
        self.t
    }
}
