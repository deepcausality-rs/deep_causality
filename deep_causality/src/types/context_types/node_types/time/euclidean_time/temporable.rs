/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{EuclideanTime, Temporal, TimeScale};

impl Temporal<f64> for EuclideanTime {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> f64 {
        self.time_unit
    }
}
