// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{DiscreteTime, Temporal, TimeScale};

impl Temporal<u64> for DiscreteTime {
    fn time_scale(&self) -> TimeScale {
        self.tick_scale
    }

    fn time_unit(&self) -> u64 {
        self.tick_unit
    }
}
