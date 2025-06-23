// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{AdjustableTime, TimeScale};
use crate::traits::contextuable::temporal::Temporal;
impl Temporal<u64> for AdjustableTime {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> &u64 {
        &self.time_unit
    }
}
