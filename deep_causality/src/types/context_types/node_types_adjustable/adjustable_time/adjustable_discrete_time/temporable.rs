// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableDiscreteTime, Temporal, TimeScale};

impl Temporal<u64> for AdjustableDiscreteTime {
    fn time_scale(&self) -> TimeScale {
        self.tick_scale
    }

    fn time_unit(&self) -> u64 {
        self.tick_unit
    }
}
