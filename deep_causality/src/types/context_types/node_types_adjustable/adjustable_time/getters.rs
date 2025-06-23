// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableTime, TimeScale};

impl AdjustableTime {
    pub fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    pub fn time_unit(&self) -> u64 {
        self.time_unit
    }
}
