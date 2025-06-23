// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableEntropicTime, Temporal, TimeScale};

impl Temporal<u64> for AdjustableEntropicTime {
    fn time_scale(&self) -> TimeScale {
        TimeScale::NoScale 
    }

    fn time_unit(&self) -> u64 {
        self.entropy_tick
    }
}
